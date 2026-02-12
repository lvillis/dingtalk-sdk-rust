use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{
    auth::AppCredentials,
    client::async_client::Client,
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

/// Async enterprise robot service.
#[derive(Clone)]
pub struct EnterpriseService {
    client: Client,
    credentials: AppCredentials,
    robot_code: String,
    access_token_cache: Option<AccessTokenCache>,
}

impl EnterpriseService {
    pub(crate) fn new(
        client: Client,
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
    pub async fn get_access_token(&self) -> Result<String> {
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
            .send_json::<GetTokenResponse>()
            .await?;

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

    async fn post_topapi_result<T, B>(&self, segments: &[&str], body: &B) -> Result<T>
    where
        T: DeserializeOwned,
        B: serde::Serialize + ?Sized,
    {
        let access_token = self.get_access_token().await?;
        let endpoint = self.client.webhook_endpoint(segments)?;
        let response = self
            .client
            .webhook_http()
            .post(endpoint.as_str())
            .query_pair("access_token", access_token)
            .json(body)?
            .send_json::<TopApiResultResponse<T>>()
            .await?;

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

    async fn post_topapi_value<B>(&self, segments: &[&str], body: &B) -> Result<Value>
    where
        B: serde::Serialize + ?Sized,
    {
        self.post_topapi_result(segments, body).await
    }

    async fn post_topapi_unit<B>(&self, segments: &[&str], body: &B) -> Result<()>
    where
        B: serde::Serialize + ?Sized,
    {
        let access_token = self.get_access_token().await?;
        let endpoint = self.client.webhook_endpoint(segments)?;
        let response = self
            .client
            .webhook_http()
            .post(endpoint.as_str())
            .query_pair("access_token", access_token)
            .json(body)?
            .send_json::<TopApiSimpleResponse>()
            .await?;

        if response.errcode != 0 {
            return Err(api_error(
                response.errcode,
                response.errmsg,
                response.request_id,
            ));
        }

        Ok(())
    }

    async fn send_enterprise_message<T: serde::Serialize + ?Sized>(
        &self,
        segments: &[&str],
        payload: &T,
    ) -> Result<String> {
        let access_token = self.get_access_token().await?;
        let endpoint = self.client.enterprise_endpoint(segments)?;

        let response = self
            .client
            .enterprise_http()
            .post(endpoint.as_str())
            .try_header("x-acs-dingtalk-access-token", &access_token)?
            .json(payload)?
            .send()
            .await?;

        let body = response.text_lossy();
        crate::transport::validate_standard_api_response(&body)?;
        Ok(body)
    }

    /// Sends a group message to a conversation.
    pub async fn send_group_message(
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
            .await
    }

    /// Sends a one-to-one message to a user.
    pub async fn send_oto_message(&self, user_id: &str, title: &str, text: &str) -> Result<String> {
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
            .await
    }

    /// Gets user details by user id.
    pub async fn contact_get_user(&self, request: ContactGetUserRequest) -> Result<Value> {
        self.post_topapi_value(&["topapi", "v2", "user", "get"], &request)
            .await
    }

    /// Gets user details by mobile.
    pub async fn contact_get_user_by_mobile(
        &self,
        request: ContactGetUserByMobileRequest,
    ) -> Result<Value> {
        self.post_topapi_value(&["topapi", "v2", "user", "getbymobile"], &request)
            .await
    }

    /// Gets user details by union id.
    pub async fn contact_get_user_by_unionid(
        &self,
        request: ContactGetUserByUnionIdRequest,
    ) -> Result<Value> {
        self.post_topapi_value(&["topapi", "user", "getbyunionid"], &request)
            .await
    }

    /// Lists users in a department.
    pub async fn contact_list_users(&self, request: ContactListUsersRequest) -> Result<Value> {
        self.post_topapi_value(&["topapi", "v2", "user", "list"], &request)
            .await
    }

    /// Creates a user.
    pub async fn contact_create_user(&self, request: ContactCreateUserRequest) -> Result<Value> {
        self.post_topapi_value(&["topapi", "v2", "user", "create"], &request)
            .await
    }

    /// Updates a user.
    pub async fn contact_update_user(&self, request: ContactUpdateUserRequest) -> Result<()> {
        self.post_topapi_unit(&["topapi", "v2", "user", "update"], &request)
            .await
    }

    /// Deletes a user.
    pub async fn contact_delete_user(&self, request: ContactDeleteUserRequest) -> Result<()> {
        self.post_topapi_unit(&["topapi", "v2", "user", "delete"], &request)
            .await
    }

    /// Gets department details.
    pub async fn contact_get_department(
        &self,
        request: ContactGetDepartmentRequest,
    ) -> Result<Value> {
        self.post_topapi_value(&["topapi", "v2", "department", "get"], &request)
            .await
    }

    /// Lists child departments.
    pub async fn contact_list_sub_departments(
        &self,
        request: ContactListSubDepartmentsRequest,
    ) -> Result<Value> {
        self.post_topapi_value(&["topapi", "v2", "department", "listsub"], &request)
            .await
    }

    /// Lists child department ids.
    pub async fn contact_list_sub_department_ids(
        &self,
        request: ContactListSubDepartmentIdsRequest,
    ) -> Result<Value> {
        self.post_topapi_value(&["topapi", "v2", "department", "listsubid"], &request)
            .await
    }

    /// Creates a department.
    pub async fn contact_create_department(
        &self,
        request: ContactCreateDepartmentRequest,
    ) -> Result<Value> {
        self.post_topapi_value(&["topapi", "v2", "department", "create"], &request)
            .await
    }

    /// Updates a department.
    pub async fn contact_update_department(
        &self,
        request: ContactUpdateDepartmentRequest,
    ) -> Result<()> {
        self.post_topapi_unit(&["topapi", "v2", "department", "update"], &request)
            .await
    }

    /// Deletes a department.
    pub async fn contact_delete_department(
        &self,
        request: ContactDeleteDepartmentRequest,
    ) -> Result<()> {
        self.post_topapi_unit(&["topapi", "v2", "department", "delete"], &request)
            .await
    }

    /// Creates an approval process instance and returns its id.
    pub async fn approval_create_process_instance(
        &self,
        request: ApprovalCreateProcessInstanceRequest,
    ) -> Result<String> {
        let access_token = self.get_access_token().await?;
        let endpoint = self
            .client
            .webhook_endpoint(&["topapi", "processinstance", "create"])?;
        let response = self
            .client
            .webhook_http()
            .post(endpoint.as_str())
            .query_pair("access_token", access_token)
            .json(&request)?
            .send_json::<ApprovalCreateProcessInstanceResponse>()
            .await?;

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
    pub async fn approval_get_process_instance(&self, process_instance_id: &str) -> Result<Value> {
        let access_token = self.get_access_token().await?;
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
            .send_json::<ApprovalGetProcessInstanceResponse>()
            .await?;

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
    pub async fn approval_list_process_instance_ids(
        &self,
        request: ApprovalListProcessInstanceIdsRequest,
    ) -> Result<ApprovalListProcessInstanceIdsResult> {
        self.post_topapi_result(&["topapi", "processinstance", "listids"], &request)
            .await
    }

    /// Terminates an approval process instance.
    pub async fn approval_terminate_process_instance(
        &self,
        request: ApprovalTerminateProcessInstanceRequest,
    ) -> Result<()> {
        let body = serde_json::json!({ "request": request });
        self.post_topapi_unit(&["topapi", "process", "instance", "terminate"], &body)
            .await
    }

    /// Replies to an incoming callback message.
    ///
    /// For private chats, this sends OTO message to `senderStaffId`;
    /// for group chats, it sends a group message to `conversationId`.
    pub async fn reply_message(
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
                .await
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
                .await
        }
    }
}
