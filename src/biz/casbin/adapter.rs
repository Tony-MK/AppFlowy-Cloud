use crate::biz::casbin::access_control::{Action, ObjectType, ToCasbinAction};
use async_trait::async_trait;
use casbin::error::AdapterError;
use casbin::Adapter;
use casbin::Filter;
use casbin::Model;
use casbin::Result;
use database::collab::select_collab_member_access_level;
use database::pg_row::AFCollabMemerAccessLevelRow;
use database::pg_row::AFWorkspaceMemberPermRow;
use database::workspace::select_workspace_member_perm_stream;
use database_entity::dto::AFAccessLevel;
use futures_util::stream::BoxStream;
use sqlx::PgPool;
use tokio_stream::StreamExt;

/// Implmentation of [`casbin::Adapter`] for access control authorisation.
/// Access control policies that are managed by workspace and collab CRUD.
pub struct PgAdapter {
  pg_pool: PgPool,
}

impl PgAdapter {
  pub fn new(pg_pool: PgPool) -> Self {
    Self { pg_pool }
  }
}

async fn create_collab_policies(
  mut stream: BoxStream<'_, sqlx::Result<AFCollabMemerAccessLevelRow>>,
) -> Result<Vec<Vec<String>>> {
  let mut policies: Vec<Vec<String>> = Vec::new();

  while let Some(result) = stream.next().await {
    let member_access_lv = result.map_err(|err| AdapterError(Box::new(err)))?;
    let policy = [
      member_access_lv.uid.to_string(),
      ObjectType::Collab(&member_access_lv.oid).to_object_id(),
      member_access_lv.access_level.to_action(),
    ]
    .to_vec();
    policies.push(policy);
  }

  Ok(policies)
}

async fn create_workspace_policies(
  mut stream: BoxStream<'_, sqlx::Result<AFWorkspaceMemberPermRow>>,
) -> Result<Vec<Vec<String>>> {
  let mut policies: Vec<Vec<String>> = Vec::new();

  while let Some(result) = stream.next().await {
    let member_permission = result.map_err(|err| AdapterError(Box::new(err)))?;
    let policy = [
      member_permission.uid.to_string(),
      ObjectType::Workspace(&member_permission.workspace_id.to_string()).to_object_id(),
      member_permission.role.to_action(),
    ]
    .to_vec();
    policies.push(policy);
  }

  Ok(policies)
}

#[async_trait]
impl Adapter for PgAdapter {
  async fn load_policy(&mut self, model: &mut dyn Model) -> Result<()> {
    let workspace_member_perm_stream = select_workspace_member_perm_stream(&self.pg_pool);
    let workspace_policies = create_workspace_policies(workspace_member_perm_stream).await?;

    // Policy definition `p` of type `p`. See `model.conf`
    model.add_policies("p", "p", workspace_policies);

    let collab_member_access_lv_stream = select_collab_member_access_level(&self.pg_pool);
    let collab_policies = create_collab_policies(collab_member_access_lv_stream).await?;

    // Policy definition `p` of type `p`. See `model.conf`
    model.add_policies("p", "p", collab_policies);

    // Grouping definition of access level to action.
    let af_access_levels = [
      AFAccessLevel::ReadOnly,
      AFAccessLevel::ReadAndComment,
      AFAccessLevel::ReadAndWrite,
      AFAccessLevel::FullAccess,
    ];
    let mut grouping_policies = Vec::new();
    for level in af_access_levels {
      // All levels can read
      grouping_policies.push([i32::from(level).to_string(), Action::Read.to_action()].to_vec());
      if level.can_write() {
        grouping_policies.push([i32::from(level).to_string(), Action::Write.to_action()].to_vec());
      }
      if level.can_delete() {
        grouping_policies.push([i32::from(level).to_string(), Action::Delete.to_action()].to_vec());
      }
    }

    // Grouping definition `g` of type `g`. See `model.conf`
    model.add_policies("g", "g", grouping_policies);

    Ok(())
  }

  async fn load_filtered_policy<'a>(&mut self, m: &mut dyn Model, _f: Filter<'a>) -> Result<()> {
    // No support for filtered.
    self.load_policy(m).await
  }
  async fn save_policy(&mut self, _m: &mut dyn Model) -> Result<()> {
    // unimplemented!()
    //
    // Adapter is used only for loading policies from database
    // since policies are managed by workspace and collab CRUD.
    Ok(())
  }
  async fn clear_policy(&mut self) -> Result<()> {
    // unimplemented!()
    //
    // Adapter is used only for loading policies from database
    // since policies are managed by workspace and collab CRUD.
    Ok(())
  }
  fn is_filtered(&self) -> bool {
    // No support for filtered.
    false
  }
  async fn add_policy(&mut self, _sec: &str, _ptype: &str, _rule: Vec<String>) -> Result<bool> {
    // unimplemented!()
    //
    // Adapter is used only for loading policies from database
    // since policies are managed by workspace and collab CRUD.
    Ok(true)
  }
  async fn add_policies(
    &mut self,
    _sec: &str,
    _ptype: &str,
    _rules: Vec<Vec<String>>,
  ) -> Result<bool> {
    // unimplemented!()
    //
    // Adapter is used only for loading policies from database
    // since policies are managed by workspace and collab CRUD.
    Ok(true)
  }
  async fn remove_policy(&mut self, _sec: &str, _ptype: &str, _rule: Vec<String>) -> Result<bool> {
    // unimplemented!()
    //
    // Adapter is used only for loading policies from database
    // since policies are managed by workspace and collab CRUD.
    Ok(true)
  }
  async fn remove_policies(
    &mut self,
    _sec: &str,
    _ptype: &str,
    _rules: Vec<Vec<String>>,
  ) -> Result<bool> {
    // unimplemented!()
    //
    // Adapter is used only for loading policies from database
    // since policies are managed by workspace and collab CRUD.
    Ok(true)
  }
  async fn remove_filtered_policy(
    &mut self,
    _sec: &str,
    _ptype: &str,
    _field_index: usize,
    _field_values: Vec<String>,
  ) -> Result<bool> {
    // unimplemented!()
    //
    // Adapter is used only for loading policies from database
    // since policies are managed by workspace and collab CRUD.
    Ok(true)
  }
}
