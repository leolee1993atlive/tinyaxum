use crate::user::model::user::User;

/// 自动注入审计字段的 trait
pub trait Auditable {
    /// 设置创建人信息
    fn set_created_by(&mut self, user: &User);
    
    /// 设置更新人信息
    fn set_updated_by(&mut self, user: &User);
    
    /// 设置删除人信息
    fn set_deleted_by(&mut self, user: &User);
}

/// 提供便捷方法的 trait，用于在服务层自动注入
pub trait AutoInjectable {
    /// 自动注入创建人信息
    fn auto_inject_create(&mut self, user: &User);
    
    /// 自动注入更新人信息
    fn auto_inject_update(&mut self, user: &User);
    
    /// 自动注入删除人信息
    fn auto_inject_delete(&mut self, user: &User);
}