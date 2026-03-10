use serde::de::DeserializeOwned;

use super::is_private_conversation;

use crate::{
    auth::AppCredentials,
    client::blocking_client::BlockingClient,
    error::{Error, Result},
    transport::{
        DEFAULT_MSG_KEY, parse_approval_create_response, parse_approval_get_response,
        parse_get_token_response, parse_standard_api_text_response, parse_topapi_result_response,
        parse_topapi_unit_response,
    },
    types::{
        ApprovalCreateProcessInstanceRequest, ApprovalListProcessInstanceIdsRequest,
        ApprovalListProcessInstanceIdsResult, ApprovalProcessInstance,
        ApprovalTerminateProcessInstanceRequest, ContactCreateDepartmentRequest,
        ContactCreateDepartmentResult, ContactCreateUserRequest, ContactCreateUserResult,
        ContactDeleteDepartmentRequest, ContactDeleteUserRequest, ContactDepartment,
        ContactGetDepartmentRequest, ContactGetUserByMobileRequest, ContactGetUserByUnionIdRequest,
        ContactGetUserRequest, ContactListSubDepartmentIdsRequest,
        ContactListSubDepartmentIdsResult, ContactListSubDepartmentsRequest,
        ContactListSubDepartmentsResult, ContactListUsersRequest, ContactListUsersResult,
        ContactUpdateDepartmentRequest, ContactUpdateUserRequest, ContactUser,
        internal::{GroupMessageRequest, MsgParam, OtoMessageRequest},
    },
};

/// Blocking enterprise robot service.
#[derive(Clone)]
pub struct BlockingEnterpriseService {
    client: BlockingClient,
    credentials: AppCredentials,
    robot_code: String,
}

impl BlockingEnterpriseService {
    pub(crate) fn new(
        client: BlockingClient,
        appkey: impl Into<String>,
        appsecret: impl Into<String>,
        robot_code: impl Into<String>,
    ) -> Self {
        Self {
            client,
            credentials: AppCredentials::new(appkey, appsecret),
            robot_code: robot_code.into(),
        }
    }

    /// Retrieves enterprise access token and refreshes cache when needed.
    pub fn get_access_token(&self) -> Result<String> {
        if let Some(token) = self.client.cached_access_token(&self.credentials) {
            return Ok(token);
        }

        let endpoint = self.client.webhook_endpoint(&["gettoken"])?;
        let payload = parse_get_token_response(
            self.client
                .webhook_http()
                .get(endpoint.as_str())
                .query_pair("appkey", self.credentials.appkey().to_string())
                .query_pair("appsecret", self.credentials.appsecret().to_string())
                .send_response()?,
            self.client.body_snippet(),
        )?;
        let access_token = payload.token;

        self.client
            .store_access_token(&self.credentials, access_token.clone(), payload.expires_in);

        Ok(access_token)
    }

    fn post_topapi_result<T, B>(&self, segments: &[&str], body: &B) -> Result<T>
    where
        T: DeserializeOwned,
        B: serde::Serialize + ?Sized,
    {
        let access_token = self.get_access_token()?;
        let endpoint = self.client.webhook_endpoint(segments)?;
        parse_topapi_result_response(
            self.client
                .webhook_http()
                .post(endpoint.as_str())
                .query_pair("access_token", access_token)
                .json(body)?
                .send_response()?,
            self.client.body_snippet(),
        )
    }

    fn post_topapi_unit<B>(&self, segments: &[&str], body: &B) -> Result<()>
    where
        B: serde::Serialize + ?Sized,
    {
        let access_token = self.get_access_token()?;
        let endpoint = self.client.webhook_endpoint(segments)?;
        parse_topapi_unit_response(
            self.client
                .webhook_http()
                .post(endpoint.as_str())
                .query_pair("access_token", access_token)
                .json(body)?
                .send_response()?,
            self.client.body_snippet(),
        )
    }

    fn send_enterprise_message<T: serde::Serialize + ?Sized>(
        &self,
        segments: &[&str],
        payload: &T,
    ) -> Result<String> {
        let access_token = self.get_access_token()?;
        let endpoint = self.client.enterprise_endpoint(segments)?;

        parse_standard_api_text_response(
            self.client
                .enterprise_http()
                .post(endpoint.as_str())
                .try_header("x-acs-dingtalk-access-token", &access_token)?
                .json(payload)?
                .send_response()?,
            self.client.body_snippet(),
        )
    }

    /// Sends a group message to a conversation.
    pub fn send_group_message(
        &self,
        open_conversation_id: &str,
        title: &str,
        text: &str,
    ) -> Result<String> {
        let request = GroupMessageRequest {
            msg_param: MsgParam {
                title: title.to_string(),
                text: text.to_string(),
            },
            msg_key: DEFAULT_MSG_KEY,
            robot_code: &self.robot_code,
            open_conversation_id,
        };

        self.send_enterprise_message(&["v1.0", "robot", "groupMessages", "send"], &request)
    }

    /// Sends a one-to-one message to a user.
    pub fn send_oto_message(&self, user_id: &str, title: &str, text: &str) -> Result<String> {
        let request = OtoMessageRequest {
            msg_param: MsgParam {
                title: title.to_string(),
                text: text.to_string(),
            },
            msg_key: DEFAULT_MSG_KEY,
            robot_code: &self.robot_code,
            user_ids: vec![user_id],
        };

        self.send_enterprise_message(&["v1.0", "robot", "oToMessages", "batchSend"], &request)
    }

    /// Gets user details by user id.
    pub fn contact_get_user(&self, request: ContactGetUserRequest) -> Result<ContactUser> {
        self.post_topapi_result(&["topapi", "v2", "user", "get"], &request)
    }

    /// Gets user details by mobile.
    pub fn contact_get_user_by_mobile(
        &self,
        request: ContactGetUserByMobileRequest,
    ) -> Result<ContactUser> {
        self.post_topapi_result(&["topapi", "v2", "user", "getbymobile"], &request)
    }

    /// Gets user details by union id.
    pub fn contact_get_user_by_unionid(
        &self,
        request: ContactGetUserByUnionIdRequest,
    ) -> Result<ContactUser> {
        self.post_topapi_result(&["topapi", "user", "getbyunionid"], &request)
    }

    /// Lists users in a department.
    pub fn contact_list_users(
        &self,
        request: ContactListUsersRequest,
    ) -> Result<ContactListUsersResult> {
        self.post_topapi_result(&["topapi", "v2", "user", "list"], &request)
    }

    /// Creates a user.
    pub fn contact_create_user(
        &self,
        request: ContactCreateUserRequest,
    ) -> Result<ContactCreateUserResult> {
        self.post_topapi_result(&["topapi", "v2", "user", "create"], &request)
    }

    /// Updates a user.
    pub fn contact_update_user(&self, request: ContactUpdateUserRequest) -> Result<()> {
        self.post_topapi_unit(&["topapi", "v2", "user", "update"], &request)
    }

    /// Deletes a user.
    pub fn contact_delete_user(&self, request: ContactDeleteUserRequest) -> Result<()> {
        self.post_topapi_unit(&["topapi", "v2", "user", "delete"], &request)
    }

    /// Gets department details.
    pub fn contact_get_department(
        &self,
        request: ContactGetDepartmentRequest,
    ) -> Result<ContactDepartment> {
        self.post_topapi_result(&["topapi", "v2", "department", "get"], &request)
    }

    /// Lists child departments.
    pub fn contact_list_sub_departments(
        &self,
        request: ContactListSubDepartmentsRequest,
    ) -> Result<ContactListSubDepartmentsResult> {
        self.post_topapi_result(&["topapi", "v2", "department", "listsub"], &request)
    }

    /// Lists child department ids.
    pub fn contact_list_sub_department_ids(
        &self,
        request: ContactListSubDepartmentIdsRequest,
    ) -> Result<ContactListSubDepartmentIdsResult> {
        self.post_topapi_result(&["topapi", "v2", "department", "listsubid"], &request)
    }

    /// Creates a department.
    pub fn contact_create_department(
        &self,
        request: ContactCreateDepartmentRequest,
    ) -> Result<ContactCreateDepartmentResult> {
        self.post_topapi_result(&["topapi", "v2", "department", "create"], &request)
    }

    /// Updates a department.
    pub fn contact_update_department(&self, request: ContactUpdateDepartmentRequest) -> Result<()> {
        self.post_topapi_unit(&["topapi", "v2", "department", "update"], &request)
    }

    /// Deletes a department.
    pub fn contact_delete_department(&self, request: ContactDeleteDepartmentRequest) -> Result<()> {
        self.post_topapi_unit(&["topapi", "v2", "department", "delete"], &request)
    }

    /// Creates an approval process instance and returns its id.
    pub fn approval_create_process_instance(
        &self,
        request: ApprovalCreateProcessInstanceRequest,
    ) -> Result<String> {
        let access_token = self.get_access_token()?;
        let endpoint = self
            .client
            .webhook_endpoint(&["topapi", "processinstance", "create"])?;
        parse_approval_create_response(
            self.client
                .webhook_http()
                .post(endpoint.as_str())
                .query_pair("access_token", access_token)
                .json(&request)?
                .send_response()?,
            self.client.body_snippet(),
        )
    }

    /// Gets approval process instance details.
    pub fn approval_get_process_instance(
        &self,
        process_instance_id: &str,
    ) -> Result<ApprovalProcessInstance> {
        let access_token = self.get_access_token()?;
        let endpoint = self
            .client
            .webhook_endpoint(&["topapi", "processinstance", "get"])?;
        let request = serde_json::json!({
            "process_instance_id": process_instance_id
        });
        parse_approval_get_response(
            self.client
                .webhook_http()
                .post(endpoint.as_str())
                .query_pair("access_token", access_token)
                .json(&request)?
                .send_response()?,
            self.client.body_snippet(),
        )
    }

    /// Lists approval process instance ids.
    pub fn approval_list_process_instance_ids(
        &self,
        request: ApprovalListProcessInstanceIdsRequest,
    ) -> Result<ApprovalListProcessInstanceIdsResult> {
        self.post_topapi_result(&["topapi", "processinstance", "listids"], &request)
    }

    /// Terminates an approval process instance.
    pub fn approval_terminate_process_instance(
        &self,
        request: ApprovalTerminateProcessInstanceRequest,
    ) -> Result<()> {
        let body = serde_json::json!({ "request": request });
        self.post_topapi_unit(&["topapi", "process", "instance", "terminate"], &body)
    }

    /// Replies to an incoming callback message.
    ///
    /// For private chats, this sends OTO message to `senderStaffId`;
    /// for group chats, it sends a group message to `conversationId`.
    pub fn reply_message(
        &self,
        data: &serde_json::Value,
        title: &str,
        text: &str,
    ) -> Result<String> {
        let msg_param = MsgParam {
            title: title.to_string(),
            text: text.to_string(),
        };

        if is_private_conversation(data) {
            let sender_staff_id = data
                .get("senderStaffId")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::InvalidConfig {
                    message: "Missing senderStaffId".to_string(),
                    source: None,
                })?;

            let request = OtoMessageRequest {
                msg_param,
                msg_key: DEFAULT_MSG_KEY,
                robot_code: &self.robot_code,
                user_ids: vec![sender_staff_id],
            };

            self.send_enterprise_message(&["v1.0", "robot", "oToMessages", "batchSend"], &request)
        } else {
            let conversation_id = data
                .get("conversationId")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::InvalidConfig {
                    message: "Missing conversationId".to_string(),
                    source: None,
                })?;

            let request = GroupMessageRequest {
                msg_param,
                msg_key: DEFAULT_MSG_KEY,
                robot_code: &self.robot_code,
                open_conversation_id: conversation_id,
            };

            self.send_enterprise_message(&["v1.0", "robot", "groupMessages", "send"], &request)
        }
    }
}
