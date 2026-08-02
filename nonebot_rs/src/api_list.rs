/// This macro lists all Onebot APIs exactly once.
/// It calls the helper macros `api_no_resp!` and `api_resp!` which
/// must be defined in the calling scope with the appropriate implementations.
#[macro_export]
macro_rules! onebot_apis {
    () => {
        // No-response APIs
        api_no_resp!(delete_msg, DeleteMsg, message_id: i32);
        api_no_resp!(send_like, SendLike, user_id: String, times: u8);
        api_no_resp!(set_group_kick, SetGroupKick, group_id: String, user_id: String, reject_add_request: bool);
        api_no_resp!(set_group_ban, SetGroupBan, group_id: String, user_id: String, duration: i64);
        api_no_resp!(set_group_anonymous_ban, SetGroupAnonymousBan, group_id: String, anonymous: $crate::event::Anonymous, flag: String, duration: i64);
        api_no_resp!(set_group_whole_ban, SetGroupWholeBan, group_id: String, enable: bool);
        api_no_resp!(set_group_admin, SetGroupAdmin, group_id: String, user_id: String, enable: bool);
        api_no_resp!(set_group_anonymous, SetGroupAnonymous, group_id: String, enable: bool);
        api_no_resp!(set_group_card, SetGroupCard, group_id: String, user_id: String, card: String);
        api_no_resp!(set_group_name, SetGroupName, group_id: String, group_name: String);
        api_no_resp!(set_group_leave, SetGroupLeave, group_id: String, is_dismiss: bool);
        api_no_resp!(set_group_special_title, SetGroupSpecialTitle, group_id: String, user_id: String, special_title: String, duration: i64);
        api_no_resp!(set_friend_add_request, SetFriendAddRequest, flag: String, approve: bool, remark: String);
        api_no_resp!(set_group_add_request, SetGroupAddRequest, flag: String, sub_type: String, approve: bool, reason: String);
        api_no_resp!(set_restart, SetRestart, delay: i64);

        // Response APIs
        api_resp!(send_msg, SendMsg, MessageId, $crate::api_resp::RespMessageId, message_type: Option<String>, user_id: Option<String>, group_id: Option<String>, message: Vec<$crate::Message>, auto_escape: bool);
        api_resp!(get_msg, GetMsg, Message, $crate::api_resp::RespMessage, message_id: i32);
        api_resp!(get_forward_msg, GetForwardMsg, Message, $crate::api_resp::RespMessage, id: String);
        api_resp!(get_login_info, GetLoginInfo, LoginInfo, $crate::api_resp::RespLoginInfo,);
        api_resp!(get_stranger_info, GetStrangerInfo, StrangerInfo, $crate::api_resp::RespStrangerInfo, user_id: String, no_cache: bool);
        api_resp!(get_friend_list, GetFriendList, FriendList, Vec<$crate::api_resp::RespFriendListItem>,);
        api_resp!(get_group_info, GetGroupInfo, GroupInfo, $crate::api_resp::RespGroupInfo, group_id: String, no_cache: bool);
        api_resp!(get_group_list, GetGroupList, GroupList, Vec<$crate::api_resp::RespGroupListItem>,);
        api_resp!(get_group_member_info, GetGroupMemberInfo, GroupMemberInfo, $crate::api_resp::RespGroupMemberInfo, group_id: String, user_id: String, no_cache: bool);
        api_resp!(get_group_member_list, GetGroupMemberList, GroupMemberList, Vec<$crate::api_resp::RespGroupMember>, group_id: String);
        api_resp!(get_group_honor_info, GetGroupHonorInfo, GroupHonorInfo, $crate::api_resp::RespGroupHonorInfo, group_id: String, type_: String);
        api_resp!(get_cookies, GetCookies, Cookies, $crate::api_resp::RespCookies, domain: String);
        api_resp!(get_csrf_token, GetCsrfToken, ScrfToken, $crate::api_resp::RespScrfToken,);
        api_resp!(get_credentials, GetCookies, Credentials, $crate::api_resp::RespCredentials, domain: String);
        api_resp!(get_record, GetRecord, File, $crate::api_resp::RespFile, file: String, out_format: String);
        api_resp!(get_image, GetImage, File, $crate::api_resp::RespFile, file: String);
        api_resp!(can_send_record, CanSendRecord, SendCheck, $crate::api_resp::RespSendCheck,);
        api_resp!(can_send_image, CanSendImage, SendCheck, $crate::api_resp::RespSendCheck,);
        api_resp!(get_status, GetStatus, Status, $crate::event::Status,);
        api_resp!(get_version_info, GetVersionInfo, VersionInfo, $crate::api_resp::RespVersionInfo,);
    };
}
