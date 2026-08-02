//! 定时任务调度插件
//!
//! 基于 `tokio-cron-scheduler` 实现，支持两种使用方式：
//!
//! # 方式一：配置文件驱动
//!
//! 在 `Nonebotrs.toml` 中声明定时任务，然后在代码中遍历配置来创建 Job：
//!
//! ```toml
//! [scheduler]
//! disable = false
//!
//! [scheduler.jobs.morning]
//! cron = "0 8 * * *"          # 每天早上8点
//! group_id = "123456"
//! message = "早上好！"
//!
//! [scheduler.jobs.reminder]
//! cron = "0 */2 * * *"        # 每2小时
//! group_id = "123456"
//! message = "该喝水了！"
//! ```
//!
//! ```rust,ignore
//! let mut scheduler = builtin_plugins::scheduler::Scheduler::new();
//!
//! // 从配置加载后，遍历 jobs 手动创建 Job
//! for (name, job_cfg) in scheduler.jobs() {
//!     if let (Some(gid), Some(msg)) = (
//!         job_cfg.get_str("group_id"),
//!         job_cfg.get_str("message"),
//!     ) {
//!         let gid = gid.to_owned();
//!         let msg = msg.to_owned();
//!         scheduler.add_cron_job(
//!             &job_cfg.cron,
//!             move || {
//!                 // 在同步回调中用 tokio::spawn 执行异步操作
//!                 let gid = gid.clone();
//!                 let msg = msg.clone();
//!                 tokio::spawn(async move {
//!                     tracing::info!("定时任务触发: 向群 {} 发送 {}", gid, msg);
//!                     // bot.send_group_msg(&gid, ...).await;
//!                 });
//!             },
//!         );
//!     }
//! }
//!
//! nb.add_plugin(scheduler);
//! ```
//!
//! # 方式二：纯代码注册
//!
//! 不依赖配置文件，直接在代码中添加定时任务：
//!
//! ```rust,ignore
//! let mut scheduler = builtin_plugins::scheduler::Scheduler::new();
//!
//! // 每5分钟执行一次
//! scheduler.add_cron_job("*/5 * * * *", || {
//!     tracing::info!("每5分钟触发一次");
//! });
//!
//! // 每天9点执行
//! scheduler.add_cron_job("0 9 * * *", || {
//!     tracing::info!("每天早上9点触发");
//! });
//!
//! nb.add_plugin(scheduler);
//! ```
//!
//! # 方式三：混合使用
//!
//! 既从配置加载，也在代码中额外添加：
//!
//! ```rust,ignore
//! let mut scheduler = builtin_plugins::scheduler::Scheduler::new();
//!
//! // 配置文件中定义的 jobs 会在 load_config() 中自动加载到 scheduler.jobs()
//! // 然后可以在注册 Plugin 之前遍历它们来创建 Job
//!
//! // 额外添加一个纯代码的 job
//! scheduler.add_cron_job("0 0 * * *", || {
//!     tracing::info!("午夜触发");
//! });
//!
//! nb.add_plugin(scheduler);
//! ```
//!
//! # Cron 表达式语法
//!
//! ```text
//! ┌───────── 分钟 (0–59)
//! │ ┌───────── 小时 (0–23)
//! │ │ ┌───────── 日 (1–31)
//! │ │ │ ┌───────── 月 (1–12)
//! │ │ │ │ ┌───────── 星期 (0–6, 0=周日)
//! │ │ │ │ │
//! * * * * *
//! ```
//!
//! | 示例 | 含义 |
//! |------|------|
//! | `0 9 * * *` | 每天早上9:00 |
//! | `*/5 * * * *` | 每5分钟 |
//! | `0 8,12,18 * * *` | 每天8:00, 12:00, 18:00 |
//! | `0 9 * * 1-5` | 工作日早上9:00 |
//! | `0 0 1 * *` | 每月1号午夜 |

use async_trait::async_trait;
use colored::*;
use serde::Deserialize;
use std::collections::HashMap;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{event, Level};

/// Prelude for Scheduler Plugin
///
/// 提供了编写定时任务常用的类型，使用 `use builtin_plugins::scheduler::prelude::*;` 导入。
pub mod prelude {
    pub use nonebot_rs::message::Message;
    pub use tokio_cron_scheduler::Job;
}

/// 定时任务调度插件
///
/// ## 字段说明
/// - `scheduler`: `tokio_cron_scheduler::JobScheduler` 实例，管理所有定时任务的调度
/// - `config`: 从配置文件加载的调度器配置
///
/// ## 生命周期
/// 1. `Scheduler::new()` — 创建空实例
/// 2. `add_job()` / `add_cron_job()` — 程序化注册任务（在 `nb.add_plugin()` 之前）
/// 3. `load_config()` — Nonebot 启动时自动调用，加载 `Nonebotrs.toml` 中的 `[scheduler]` 小节
/// 4. `run()` — Nonebot 启动时自动调用，启动调度器开始计时
///
/// 注意：所有的 `add_*` 调用必须在 `nb.run()` 之前完成。
/// 调度器一旦通过 `run()` 启动，不能再添加新的 Job。
pub struct Scheduler {
    scheduler: JobScheduler,
    config: SchedulerConfig,
}

impl std::fmt::Debug for Scheduler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scheduler")
            .field("config", &self.config)
            .finish()
    }
}

/// 调度器配置
///
/// 对应 `Nonebotrs.toml` 中的 `[scheduler]` 小节：
///
/// ```toml
/// [scheduler]
/// disable = false
///
/// [scheduler.jobs.job_name]
/// cron = "0 9 * * *"
/// # 以下为自定义字段，可任意扩展
/// group_id = "123456"
/// message = "hello"
/// ```
#[derive(Debug, Deserialize)]
pub struct SchedulerConfig {
    /// 是否禁用整个调度器，默认 `false`
    #[serde(default)]
    pub disable: bool,
    /// 所有 Job 的配置映射，key 为 job 名称，value 为对应的 JobConfig
    /// `#[serde(flatten)]` 表示 TOML 中 `[scheduler]` 下除 `disable` 外的所有表都会被收集到此 HashMap
    #[serde(flatten)]
    pub jobs: HashMap<String, JobConfig>,
}

/// 单个定时任务的配置
///
/// 必须提供 `cron` 字段，其余字段可自由扩展。
/// 通过 `get_str()` / `get_int()` 等辅助方法读取自定义字段。
///
/// ```toml
/// [scheduler.jobs.my_task]
/// cron = "0 9 * * *"
/// some_custom_field = "value"
/// another_field = 42
/// ```
#[derive(Debug, Deserialize, Clone)]
pub struct JobConfig {
    /// Cron 表达式，定义任务的触发时间规则
    pub cron: String,
    /// 捕获 TOML 中除 `cron`、`disable` 外的所有字段
    /// 用户可以在每个 Job 配置中自由添加自定义字段
    #[serde(flatten)]
    extra: HashMap<String, toml::Value>,
}

impl JobConfig {
    /// 获取自定义字段中的字符串值
    ///
    /// ```rust,ignore
    /// let msg = job_config.get_str("message");  // Option<&str>
    /// ```
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.extra.get(key).and_then(|v| v.as_str())
    }

    /// 获取自定义字段中的整数值
    ///
    /// ```rust,ignore
    /// let count = job_config.get_int("count");  // Option<i64>
    /// ```
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.extra.get(key).and_then(|v| v.as_integer())
    }

    /// 获取自定义字段中的布尔值
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.extra.get(key).and_then(|v| v.as_bool())
    }

    /// 获取自定义字段中的浮点数值
    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.extra.get(key).and_then(|v| v.as_float())
    }

    /// 访问完整的额外字段 HashMap
    ///
    /// 用于需要直接操作 `toml::Value` 的高级场景。
    pub fn extra(&self) -> &HashMap<String, toml::Value> {
        &self.extra
    }
}

// ============================================================
// Plugin trait 实现
// ============================================================

#[async_trait]
impl nonebot_rs::Plugin for Scheduler {
    /// 启动调度器
    ///
    /// 在 Nonebot 启动流程中自动调用。
    /// 如果 `config.disable` 为 `true`，则跳过启动。
    ///
    /// 注意：`bot_getter` 参数在此处被忽略。
    /// 如果定时任务需要发送消息，请在创建 Job 时通过闭包捕获 Bot 实例或相关 sender。
    fn run(&self, _: nonebot_rs::EventReceiver, _: nonebot_rs::BotGetter) {
        if !self.config.disable {
            // spawn 到单独的 task 中，避免阻塞 Nonebot 启动
            tokio::spawn(self.scheduler.start());
        }
    }

    fn plugin_name(&self) -> &'static str {
        "Scheduler"
    }

    /// 从 `Nonebotrs.toml` 加载调度器配置
    ///
    /// 此方法在 Nonebot 启动过程中自动调用（在 `run()` 之前）。
    /// 加载后的配置可通过 `jobs()` 方法访问。
    ///
    /// 注意：此方法只加载配置数据。如果需要根据配置创建实际的定时任务，
    /// 请在调用 `nb.add_plugin(scheduler)` 之前，
    /// 先调用 `scheduler.jobs()` 遍历配置并调用 `add_cron_job()` 注册任务。
    async fn load_config(&mut self, config: toml::Value) {
        self.config = config.try_into().expect("Scheduler load config fail");
        event!(
            Level::INFO,
            "[{}] 已加载配置，共 {} 个定时任务",
            self.plugin_name().red(),
            self.config.jobs.len(),
        );
        // 打印每个已加载任务的详情
        for (name, job_cfg) in &self.config.jobs {
            event!(
                Level::DEBUG,
                "[{}]   - {}: cron=\"{}\"",
                self.plugin_name().red(),
                name,
                job_cfg.cron,
            );
        }
    }
}

// ============================================================
// 构造与方法
// ============================================================

impl Scheduler {
    /// 创建一个空调度器实例
    ///
    /// ```rust,ignore
    /// let mut scheduler = builtin_plugins::scheduler::Scheduler::new();
    /// scheduler.add_cron_job("0 9 * * *", |_id, _lock| {
    ///     Box::pin(async move { println!("9am!"); })
    /// });
    /// nb.add_plugin(scheduler);
    /// ```
    pub fn new() -> Self {
        Scheduler {
            scheduler: JobScheduler::new(),
            config: SchedulerConfig {
                disable: false,
                jobs: HashMap::new(),
            },
        }
    }

    /// 直接添加一个 `tokio_cron_scheduler::Job` 实例
    ///
    /// 适用于需要使用 `Job` 高级特性的场景。
    /// 对于大多数场景，推荐使用更简洁的 [`add_cron_job`](Self::add_cron_job)。
    pub fn add_job(&mut self, job: Job) {
        self.scheduler.add(job).expect("Failed to add job to scheduler");
    }

    /// 根据 Cron 表达式和回调函数创建一个 Job 并添加到调度器
    ///
    /// # 参数
    /// - `cron`: Cron 表达式字符串，如 `"0 9 * * *"`
    /// - `action`: 回调函数，当 cron 触发时执行。应当是同步函数（不阻塞）。
    ///   如需执行异步操作，在回调内部使用 `tokio::spawn`。
    ///
    /// # 示例：同步回调
    ///
    /// ```rust,ignore
    /// scheduler.add_cron_job("*/5 * * * *", || {
    ///     tracing::info!("每5分钟触发一次");
    /// });
    /// ```
    ///
    /// # 示例：需要异步操作
    ///
    /// ```rust,ignore
    /// scheduler.add_cron_job("0 9 * * *", || {
    ///     tokio::spawn(async move {
    ///         // 在这里执行异步操作，如调用 Bot API
    ///         tracing::info!("9点任务执行中...");
    ///     });
    /// });
    /// ```
    ///
    /// # 示例：通过 move 闭包捕获外部数据
    ///
    /// ```rust,ignore
    /// let msg = "hello".to_string();
    /// scheduler.add_cron_job("0 9 * * *", move || {
    ///     let msg = msg.clone();
    ///     tracing::info!("定时消息: {}", msg);
    /// });
    /// ```
    pub fn add_cron_job<F>(&mut self, cron: &str, mut action: F)
    where
        F: FnMut() + Send + Sync + 'static,
    {
        let job = Job::new_cron_job(cron, move |_uuid, _lock| {
            action();
        })
        .expect("Failed to create cron job");
        self.scheduler.add(job).expect("Failed to add cron job to scheduler");
    }

    /// 访问已加载的 Job 配置映射
    ///
    /// 通常在 `nb.add_plugin()` 之前调用，遍历配置来创建实际的 Job：
    ///
    /// ```rust,ignore
    /// let mut scheduler = Scheduler::new();
    /// // ... nb.add_plugin(scheduler) 之后 load_config 被调用 ...
    ///
    /// // 但在 nb.run() 之前，无法直接访问 jobs()
    /// // 推荐模式：先创建 scheduler，手动加载配置
    /// ```
    pub fn jobs(&self) -> &HashMap<String, JobConfig> {
        &self.config.jobs
    }

    /// 检查调度器是否被禁用
    pub fn is_disabled(&self) -> bool {
        self.config.disable
    }
}
