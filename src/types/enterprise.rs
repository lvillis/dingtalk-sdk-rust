use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Request for getting a user by `userid`.
#[derive(Debug, Clone, Serialize)]
pub struct ContactGetUserRequest {
    /// DingTalk user id.
    pub userid: String,
    /// Language code (for example `zh_CN`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

impl ContactGetUserRequest {
    /// Creates a request with required fields.
    #[must_use]
    pub fn new(userid: impl Into<String>) -> Self {
        Self {
            userid: userid.into(),
            language: None,
        }
    }

    /// Sets language preference.
    #[must_use]
    pub fn language(mut self, value: impl Into<String>) -> Self {
        self.language = Some(value.into());
        self
    }
}

/// Request for getting a user by mobile number.
#[derive(Debug, Clone, Serialize)]
pub struct ContactGetUserByMobileRequest {
    /// Mobile phone number.
    pub mobile: String,
}

impl ContactGetUserByMobileRequest {
    /// Creates a request with required fields.
    #[must_use]
    pub fn new(mobile: impl Into<String>) -> Self {
        Self {
            mobile: mobile.into(),
        }
    }
}

/// Request for getting a user by union id.
#[derive(Debug, Clone, Serialize)]
pub struct ContactGetUserByUnionIdRequest {
    /// Union id.
    pub unionid: String,
}

impl ContactGetUserByUnionIdRequest {
    /// Creates a request with required fields.
    #[must_use]
    pub fn new(unionid: impl Into<String>) -> Self {
        Self {
            unionid: unionid.into(),
        }
    }
}

/// Request for listing users in a department.
#[derive(Debug, Clone, Serialize)]
pub struct ContactListUsersRequest {
    /// Department id.
    pub dept_id: i64,
    /// Cursor for pagination.
    pub cursor: i64,
    /// Page size.
    pub size: i64,
    /// Language code (for example `zh_CN`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Optional ordering field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_field: Option<String>,
    /// Whether to include access-limited users.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contain_access_limit: Option<bool>,
}

impl ContactListUsersRequest {
    /// Creates a request with required fields.
    #[must_use]
    pub fn new(dept_id: i64, cursor: i64, size: i64) -> Self {
        Self {
            dept_id,
            cursor,
            size,
            language: None,
            order_field: None,
            contain_access_limit: None,
        }
    }

    /// Sets language preference.
    #[must_use]
    pub fn language(mut self, value: impl Into<String>) -> Self {
        self.language = Some(value.into());
        self
    }

    /// Sets ordering field.
    #[must_use]
    pub fn order_field(mut self, value: impl Into<String>) -> Self {
        self.order_field = Some(value.into());
        self
    }

    /// Sets access-limit inclusion behavior.
    #[must_use]
    pub fn contain_access_limit(mut self, value: bool) -> Self {
        self.contain_access_limit = Some(value);
        self
    }
}

/// Request for creating a user.
#[derive(Debug, Clone, Serialize)]
pub struct ContactCreateUserRequest {
    /// Display name.
    pub name: String,
    /// Mobile phone number.
    pub mobile: String,
    /// Department id list encoded as DingTalk expects.
    pub dept_id_list: String,
    /// Optional user id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userid: Option<String>,
    /// Additional pass-through fields supported by DingTalk.
    #[serde(flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl ContactCreateUserRequest {
    /// Creates a request with required fields.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        mobile: impl Into<String>,
        dept_id_list: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            mobile: mobile.into(),
            dept_id_list: dept_id_list.into(),
            userid: None,
            extra: BTreeMap::new(),
        }
    }

    /// Sets explicit user id.
    #[must_use]
    pub fn userid(mut self, value: impl Into<String>) -> Self {
        self.userid = Some(value.into());
        self
    }

    /// Adds a custom extra field.
    #[must_use]
    pub fn insert_extra(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

/// Request for updating a user.
#[derive(Debug, Clone, Serialize)]
pub struct ContactUpdateUserRequest {
    /// User id.
    pub userid: String,
    /// Additional pass-through fields supported by DingTalk.
    #[serde(flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl ContactUpdateUserRequest {
    /// Creates a request with required fields.
    #[must_use]
    pub fn new(userid: impl Into<String>) -> Self {
        Self {
            userid: userid.into(),
            extra: BTreeMap::new(),
        }
    }

    /// Adds a custom extra field.
    #[must_use]
    pub fn insert_extra(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

/// Request for deleting a user.
#[derive(Debug, Clone, Serialize)]
pub struct ContactDeleteUserRequest {
    /// User id.
    pub userid: String,
}

impl ContactDeleteUserRequest {
    /// Creates a request with required fields.
    #[must_use]
    pub fn new(userid: impl Into<String>) -> Self {
        Self {
            userid: userid.into(),
        }
    }
}

/// Request for getting a department.
#[derive(Debug, Clone, Serialize)]
pub struct ContactGetDepartmentRequest {
    /// Department id.
    pub dept_id: i64,
    /// Language code (for example `zh_CN`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

impl ContactGetDepartmentRequest {
    /// Creates a request with required fields.
    #[must_use]
    pub fn new(dept_id: i64) -> Self {
        Self {
            dept_id,
            language: None,
        }
    }

    /// Sets language preference.
    #[must_use]
    pub fn language(mut self, value: impl Into<String>) -> Self {
        self.language = Some(value.into());
        self
    }
}

/// Request for creating a department.
#[derive(Debug, Clone, Serialize)]
pub struct ContactCreateDepartmentRequest {
    /// Department name.
    pub name: String,
    /// Parent department id.
    pub parent_id: i64,
    /// Additional pass-through fields supported by DingTalk.
    #[serde(flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl ContactCreateDepartmentRequest {
    /// Creates a request with required fields.
    #[must_use]
    pub fn new(name: impl Into<String>, parent_id: i64) -> Self {
        Self {
            name: name.into(),
            parent_id,
            extra: BTreeMap::new(),
        }
    }

    /// Adds a custom extra field.
    #[must_use]
    pub fn insert_extra(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

/// Request for updating a department.
#[derive(Debug, Clone, Serialize)]
pub struct ContactUpdateDepartmentRequest {
    /// Department id.
    pub dept_id: i64,
    /// New department name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New parent department id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<i64>,
    /// Additional pass-through fields supported by DingTalk.
    #[serde(flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl ContactUpdateDepartmentRequest {
    /// Creates a request with required fields.
    #[must_use]
    pub fn new(dept_id: i64) -> Self {
        Self {
            dept_id,
            name: None,
            parent_id: None,
            extra: BTreeMap::new(),
        }
    }

    /// Sets department name.
    #[must_use]
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    /// Sets parent department id.
    #[must_use]
    pub fn parent_id(mut self, value: i64) -> Self {
        self.parent_id = Some(value);
        self
    }

    /// Adds a custom extra field.
    #[must_use]
    pub fn insert_extra(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

/// Request for deleting a department.
#[derive(Debug, Clone, Serialize)]
pub struct ContactDeleteDepartmentRequest {
    /// Department id.
    pub dept_id: i64,
}

impl ContactDeleteDepartmentRequest {
    /// Creates a request with required fields.
    #[must_use]
    pub fn new(dept_id: i64) -> Self {
        Self { dept_id }
    }
}

/// Request for listing child departments.
#[derive(Debug, Clone, Serialize)]
pub struct ContactListSubDepartmentsRequest {
    /// Department id.
    pub dept_id: i64,
    /// Language code (for example `zh_CN`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

/// Request for listing child department ids.
#[derive(Debug, Clone, Serialize)]
pub struct ContactListSubDepartmentIdsRequest {
    /// Department id.
    pub dept_id: i64,
    /// Language code (for example `zh_CN`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

impl ContactListSubDepartmentIdsRequest {
    /// Creates a request with required fields.
    #[must_use]
    pub fn new(dept_id: i64) -> Self {
        Self {
            dept_id,
            language: None,
        }
    }

    /// Sets language preference.
    #[must_use]
    pub fn language(mut self, value: impl Into<String>) -> Self {
        self.language = Some(value.into());
        self
    }
}

impl ContactListSubDepartmentsRequest {
    /// Creates a request with required fields.
    #[must_use]
    pub fn new(dept_id: i64) -> Self {
        Self {
            dept_id,
            language: None,
        }
    }

    /// Sets language preference.
    #[must_use]
    pub fn language(mut self, value: impl Into<String>) -> Self {
        self.language = Some(value.into());
        self
    }
}

/// Form field item for approval instance creation.
#[derive(Debug, Clone, Serialize)]
pub struct ApprovalFormComponentValue {
    /// Form field name.
    pub name: String,
    /// Form field value.
    pub value: String,
}

impl ApprovalFormComponentValue {
    /// Creates a form field item.
    #[must_use]
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// Request for creating an approval process instance.
#[derive(Debug, Clone, Serialize)]
pub struct ApprovalCreateProcessInstanceRequest {
    /// Process code.
    pub process_code: String,
    /// Originator user id.
    pub originator_user_id: String,
    /// Department id.
    pub dept_id: i64,
    /// Optional approvers list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approvers: Option<String>,
    /// Optional cc list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc_list: Option<String>,
    /// Optional cc position.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc_position: Option<String>,
    /// Form fields.
    pub form_component_values: Vec<ApprovalFormComponentValue>,
}

impl ApprovalCreateProcessInstanceRequest {
    /// Creates a request with required fields.
    #[must_use]
    pub fn new(
        process_code: impl Into<String>,
        originator_user_id: impl Into<String>,
        dept_id: i64,
        form_component_values: Vec<ApprovalFormComponentValue>,
    ) -> Self {
        Self {
            process_code: process_code.into(),
            originator_user_id: originator_user_id.into(),
            dept_id,
            approvers: None,
            cc_list: None,
            cc_position: None,
            form_component_values,
        }
    }

    /// Sets approvers list.
    #[must_use]
    pub fn approvers(mut self, value: impl Into<String>) -> Self {
        self.approvers = Some(value.into());
        self
    }

    /// Sets cc list.
    #[must_use]
    pub fn cc_list(mut self, value: impl Into<String>) -> Self {
        self.cc_list = Some(value.into());
        self
    }

    /// Sets cc position.
    #[must_use]
    pub fn cc_position(mut self, value: impl Into<String>) -> Self {
        self.cc_position = Some(value.into());
        self
    }
}

/// Request for listing approval process instance ids.
#[derive(Debug, Clone, Serialize)]
pub struct ApprovalListProcessInstanceIdsRequest {
    /// Start timestamp in milliseconds.
    pub start_time: i64,
    /// End timestamp in milliseconds.
    pub end_time: i64,
    /// Cursor for pagination.
    pub cursor: i64,
    /// Page size.
    pub size: i64,
    /// Optional process code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_code: Option<String>,
    /// Optional user id list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userid_list: Option<String>,
}

/// Request for terminating an approval process instance.
#[derive(Debug, Clone, Serialize)]
pub struct ApprovalTerminateProcessInstanceRequest {
    /// Process instance id.
    pub process_instance_id: String,
    /// Operator user id.
    pub operating_userid: String,
    /// Whether operator is system.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_system: Option<bool>,
    /// Optional termination remark.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
}

impl ApprovalTerminateProcessInstanceRequest {
    /// Creates a request with required fields.
    #[must_use]
    pub fn new(
        process_instance_id: impl Into<String>,
        operating_userid: impl Into<String>,
    ) -> Self {
        Self {
            process_instance_id: process_instance_id.into(),
            operating_userid: operating_userid.into(),
            is_system: None,
            remark: None,
        }
    }

    /// Marks the operation as system-initiated.
    #[must_use]
    pub fn is_system(mut self, value: bool) -> Self {
        self.is_system = Some(value);
        self
    }

    /// Sets termination remark.
    #[must_use]
    pub fn remark(mut self, value: impl Into<String>) -> Self {
        self.remark = Some(value.into());
        self
    }
}

impl ApprovalListProcessInstanceIdsRequest {
    /// Creates a request with required fields.
    #[must_use]
    pub fn new(start_time: i64, end_time: i64, cursor: i64, size: i64) -> Self {
        Self {
            start_time,
            end_time,
            cursor,
            size,
            process_code: None,
            userid_list: None,
        }
    }

    /// Sets process code filter.
    #[must_use]
    pub fn process_code(mut self, value: impl Into<String>) -> Self {
        self.process_code = Some(value.into());
        self
    }

    /// Sets user id list filter.
    #[must_use]
    pub fn userid_list(mut self, value: impl Into<String>) -> Self {
        self.userid_list = Some(value.into());
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
/// Response payload for process-instance id listing.
pub struct ApprovalListProcessInstanceIdsResult {
    /// Process instance ids.
    pub list: Vec<String>,
    /// Cursor for next page.
    pub next_cursor: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
/// Typed user object from contact APIs.
pub struct ContactUser {
    /// DingTalk user id.
    #[serde(default)]
    pub userid: Option<String>,
    /// DingTalk union id.
    #[serde(default)]
    pub unionid: Option<String>,
    /// Display name.
    #[serde(default)]
    pub name: Option<String>,
    /// Mobile number.
    #[serde(default)]
    pub mobile: Option<String>,
    /// Additional response fields not modeled explicitly.
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
/// Response payload for contact user listing.
pub struct ContactListUsersResult {
    /// Whether there are more records.
    #[serde(default)]
    pub has_more: Option<bool>,
    /// Cursor for the next page.
    #[serde(default)]
    pub next_cursor: Option<i64>,
    /// User records in this page.
    #[serde(default)]
    pub list: Vec<ContactUser>,
    /// Additional response fields not modeled explicitly.
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
/// Response payload for contact user creation.
pub struct ContactCreateUserResult {
    /// Created user id.
    #[serde(default)]
    pub userid: Option<String>,
    /// Related union id when provided by DingTalk.
    #[serde(default)]
    pub unionid: Option<String>,
    /// Additional response fields not modeled explicitly.
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
/// Typed department object from contact APIs.
pub struct ContactDepartment {
    /// Department id.
    #[serde(default, alias = "id")]
    pub dept_id: Option<i64>,
    /// Department name.
    #[serde(default)]
    pub name: Option<String>,
    /// Parent department id.
    #[serde(default)]
    pub parent_id: Option<i64>,
    /// Additional response fields not modeled explicitly.
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
/// Response payload for listing child departments.
pub struct ContactListSubDepartmentsResult {
    /// Child department records.
    #[serde(default, alias = "dept_list", alias = "department", alias = "list")]
    pub departments: Vec<ContactDepartment>,
    /// Additional response fields not modeled explicitly.
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
/// Response payload for listing child department ids.
pub struct ContactListSubDepartmentIdsResult {
    /// Child department id list.
    #[serde(default, alias = "list", alias = "department_ids")]
    pub dept_id_list: Vec<i64>,
    /// Additional response fields not modeled explicitly.
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
/// Response payload for creating a department.
pub struct ContactCreateDepartmentResult {
    /// Created department id.
    #[serde(default, alias = "id")]
    pub dept_id: Option<i64>,
    /// Additional response fields not modeled explicitly.
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
/// Typed approval process instance payload.
pub struct ApprovalProcessInstance {
    /// Approval process instance id.
    #[serde(default)]
    pub process_instance_id: Option<String>,
    /// Additional response fields not modeled explicitly.
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}

#[cfg(test)]
mod tests {
    use super::{
        ApprovalProcessInstance, ApprovalTerminateProcessInstanceRequest, ContactListUsersResult,
    };

    #[test]
    fn approval_terminate_request_serializes_optional_remark() {
        let request = ApprovalTerminateProcessInstanceRequest::new("PROC-1", "user-1")
            .is_system(true)
            .remark("cancelled by sdk test");

        let value = serde_json::to_value(request).expect("request should serialize");
        assert_eq!(
            value.get("remark").and_then(serde_json::Value::as_str),
            Some("cancelled by sdk test")
        );
    }

    #[test]
    fn contact_list_users_result_parses_known_and_extra_fields() {
        let raw = r#"{
            "has_more": true,
            "next_cursor": 30,
            "list": [{"userid":"u-1","name":"Alice"}],
            "unknown_flag": 1
        }"#;
        let parsed: ContactListUsersResult =
            serde_json::from_str(raw).expect("response should deserialize");

        assert_eq!(parsed.has_more, Some(true));
        assert_eq!(parsed.next_cursor, Some(30));
        assert_eq!(parsed.list.len(), 1);
        assert_eq!(parsed.list[0].userid.as_deref(), Some("u-1"));
        assert_eq!(
            parsed
                .extra
                .get("unknown_flag")
                .and_then(serde_json::Value::as_i64),
            Some(1)
        );
    }

    #[test]
    fn approval_process_instance_parses_known_and_extra_fields() {
        let raw = r#"{"process_instance_id":"PROC-1","biz_id":"BIZ-1"}"#;
        let parsed: ApprovalProcessInstance =
            serde_json::from_str(raw).expect("response should deserialize");

        assert_eq!(parsed.process_instance_id.as_deref(), Some("PROC-1"));
        assert_eq!(
            parsed
                .extra
                .get("biz_id")
                .and_then(serde_json::Value::as_str),
            Some("BIZ-1")
        );
    }
}
