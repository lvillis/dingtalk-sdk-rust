use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{
    auth::AppCredentials,
    client::blocking_client::BlockingClient,
    error::{Error, Result},
    transport::{AccessTokenCache, DEFAULT_MSG_KEY, api_error},
    types::{
        ApprovalCreateProcessInstanceRequest, ApprovalListProcessInstanceIdsRequest,
        ApprovalListProcessInstanceIdsResult, ApprovalTerminateProcessInstanceRequest,
        ContactCreateDepartmentRequest, ContactCreateUserRequest, ContactDeleteDepartmentRequest,
        ContactDeleteUserRequest, ContactGetDepartmentRequest, ContactGetUserByMobileRequest,
        ContactGetUserByUnionIdRequest, ContactGetUserRequest, ContactListSubDepartmentIdsRequest,
        ContactListSubDepartmentsRequest, ContactListUsersRequest, ContactUpdateDepartmentRequest,
        ContactUpdateUserRequest,
        internal::{
            ApprovalCreateProcessInstanceResponse, ApprovalGetProcessInstanceResponse,
            GetTokenResponse, GroupMessageRequest, MsgParam, OtoMessageRequest,
            TopApiResultResponse, TopApiSimpleResponse,
        },
    },
};

/// Blocking enterprise robot service.
#[derive(Clone)]
pub struct BlockingEnterpriseService {
    client: BlockingClient,
    credentials: AppCredentials,
    robot_code: String,
    access_token_cache: Option<AccessTokenCache>,
}

impl BlockingEnterpriseService {
    pub(crate) fn new(
        client: BlockingClient,
        appkey: impl Into<String>,
        appsecret: impl Into<String>,
        robot_code: impl Into<String>,
    ) -> Self {
        let access_token_cache = client
            .cache_access_token_enabled()
            .then(|| AccessTokenCache::new(client.token_refresh_margin()));

        Self {
            client,
            credentials: AppCredentials::new(appkey, appsecret),
            robot_code: robot_code.into(),
            access_token_cache,
        }
    }

    /// Retrieves enterprise access token and refreshes cache when needed.
    pub fn get_access_token(&self) -> Result<String> {
        if let Some(token) = self
            .access_token_cache
            .as_ref()
            .and_then(AccessTokenCache::get)
        {
            return Ok(token);
        }

        let endpoint = self.client.webhook_endpoint(&["gettoken"])?;
        let response = self
            .client
            .webhook_http()
            .get(endpoint.as_str())
            .query_pair("appkey", self.credentials.appkey().to_string())
            .query_pair("appsecret", self.credentials.appsecret().to_string())
            .send_json::<GetTokenResponse>()?;

        if response.errcode != 0 {
            return Err(api_error(response.errcode, response.errmsg, None));
        }

        let access_token = response
            .access_token
            .ok_or_else(|| api_error(-1, "No access token returned", None))?;

        if let Some(cache) = &self.access_token_cache {
            cache.store(access_token.clone(), response.expires_in);
        }

        Ok(access_token)
    }

    fn post_topapi_result<T, B>(&self, segments: &[&str], body: &B) -> Result<T>
    where
        T: DeserializeOwned,
        B: serde::Serialize + ?Sized,
    {
        let access_token = self.get_access_token()?;
        let endpoint = self.client.webhook_endpoint(segments)?;
        let response = self
            .client
            .webhook_http()
            .post(endpoint.as_str())
            .query_pair("access_token", access_token)
            .json(body)?
            .send_json::<TopApiResultResponse<T>>()?;

        if response.errcode != 0 {
            return Err(api_error(
                response.errcode,
                response.errmsg,
                response.request_id,
            ));
        }

        response
            .result
            .ok_or_else(|| api_error(-1, "Missing result field in topapi response", None))
    }

    fn post_topapi_value<B>(&self, segments: &[&str], body: &B) -> Result<Value>
    where
        B: serde::Serialize + ?Sized,
    {
        self.post_topapi_result(segments, body)
    }

    fn post_topapi_unit<B>(&self, segments: &[&str], body: &B) -> Result<()>
    where
        B: serde::Serialize + ?Sized,
    {
        let access_token = self.get_access_token()?;
        let endpoint = self.client.webhook_endpoint(segments)?;
        let response = self
            .client
            .webhook_http()
            .post(endpoint.as_str())
            .query_pair("access_token", access_token)
            .json(body)?
            .send_json::<TopApiSimpleResponse>()?;

        if response.errcode != 0 {
            return Err(api_error(
                response.errcode,
                response.errmsg,
                response.request_id,
            ));
        }

        Ok(())
    }

    fn send_enterprise_message<T: serde::Serialize + ?Sized>(
        &self,
        segments: &[&str],
        payload: &T,
    ) -> Result<String> {
        let access_token = self.get_access_token()?;
        let endpoint = self.client.enterprise_endpoint(segments)?;

        let response = self
            .client
            .enterprise_http()
            .post(endpoint.as_str())
            .try_header("x-acs-dingtalk-access-token", &access_token)?
            .json(payload)?
            .send()?;

        let body = response.text_lossy();
        crate::transport::validate_standard_api_response(&body)?;
        Ok(body)
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
    pub fn contact_get_user(&self, request: ContactGetUserRequest) -> Result<Value> {
        self.post_topapi_value(&["topapi", "v2", "user", "get"], &request)
    }

    /// Gets user details by mobile.
    pub fn contact_get_user_by_mobile(
        &self,
        request: ContactGetUserByMobileRequest,
    ) -> Result<Value> {
        self.post_topapi_value(&["topapi", "v2", "user", "getbymobile"], &request)
    }

    /// Gets user details by union id.
    pub fn contact_get_user_by_unionid(
        &self,
        request: ContactGetUserByUnionIdRequest,
    ) -> Result<Value> {
        self.post_topapi_value(&["topapi", "user", "getbyunionid"], &request)
    }

    /// Lists users in a department.
    pub fn contact_list_users(&self, request: ContactListUsersRequest) -> Result<Value> {
        self.post_topapi_value(&["topapi", "v2", "user", "list"], &request)
    }

    /// Creates a user.
    pub fn contact_create_user(&self, request: ContactCreateUserRequest) -> Result<Value> {
        self.post_topapi_value(&["topapi", "v2", "user", "create"], &request)
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
    pub fn contact_get_department(&self, request: ContactGetDepartmentRequest) -> Result<Value> {
        self.post_topapi_value(&["topapi", "v2", "department", "get"], &request)
    }

    /// Lists child departments.
    pub fn contact_list_sub_departments(
        &self,
        request: ContactListSubDepartmentsRequest,
    ) -> Result<Value> {
        self.post_topapi_value(&["topapi", "v2", "department", "listsub"], &request)
    }

    /// Lists child department ids.
    pub fn contact_list_sub_department_ids(
        &self,
        request: ContactListSubDepartmentIdsRequest,
    ) -> Result<Value> {
        self.post_topapi_value(&["topapi", "v2", "department", "listsubid"], &request)
    }

    /// Creates a department.
    pub fn contact_create_department(
        &self,
        request: ContactCreateDepartmentRequest,
    ) -> Result<Value> {
        self.post_topapi_value(&["topapi", "v2", "department", "create"], &request)
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
        let response = self
            .client
            .webhook_http()
            .post(endpoint.as_str())
            .query_pair("access_token", access_token)
            .json(&request)?
            .send_json::<ApprovalCreateProcessInstanceResponse>()?;

        if response.errcode != 0 {
            return Err(api_error(
                response.errcode,
                response.errmsg,
                response.request_id,
            ));
        }

        response
            .process_instance_id
            .ok_or_else(|| api_error(-1, "Missing process_instance_id in response", None))
    }

    /// Gets approval process instance details.
    pub fn approval_get_process_instance(&self, process_instance_id: &str) -> Result<Value> {
        let access_token = self.get_access_token()?;
        let endpoint = self
            .client
            .webhook_endpoint(&["topapi", "processinstance", "get"])?;
        let request = serde_json::json!({
            "process_instance_id": process_instance_id
        });
        let response = self
            .client
            .webhook_http()
            .post(endpoint.as_str())
            .query_pair("access_token", access_token)
            .json(&request)?
            .send_json::<ApprovalGetProcessInstanceResponse>()?;

        if response.errcode != 0 {
            return Err(api_error(
                response.errcode,
                response.errmsg,
                response.request_id,
            ));
        }

        response
            .process_instance
            .ok_or_else(|| api_error(-1, "Missing process_instance field in response", None))
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

        if data.get("conversationType").and_then(|v| v.as_str()) == Some("1") {
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
