use chat_core::Workspace;

use crate::{error::AppError, AppState};

impl AppState {
    pub async fn find_workspace_by_name(&self, name: &str) -> Result<Option<Workspace>, AppError> {
        let workspace = sqlx::query_as(
            r#"SELECT id, name, owner_id, created_at FROM workspaces WHERE name = $1"#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(workspace)
    }

    pub async fn find_workspace_by_id(&self, id: i64) -> Result<Option<Workspace>, AppError> {
        let workspace = sqlx::query_as(
            r#"SELECT id, name, owner_id, created_at FROM workspaces WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(workspace)
    }

    // Create a new workspace
    pub async fn create_workspace(&self, name: &str, user_id: u64) -> Result<Workspace, AppError> {
        let workspace = self.find_workspace_by_name(name).await?;
        if workspace.is_some() {
            return Err(AppError::WorkspaceAlreadyExists(name.to_string()));
        }

        let workspace = sqlx::query_as(
            r#"INSERT INTO workspaces (name, owner_id) VALUES ($1, $2) RETURNING id, name, owner_id, created_at"#,
        )
        .bind(name)
        .bind(user_id as i64)
        .fetch_one(&self.pool)
        .await?;

        Ok(workspace)
    }

    pub async fn update_workspace_owner(
        &self,
        id: u64,
        owner_id: u64,
    ) -> Result<Workspace, AppError> {
        let workspace = sqlx::query_as(
            r#"
            UPDATE workspaces SET owner_id = $1
            WHERE id = $2 and (Select ws_id From users where id = $1) = $2 RETURNING id, name, owner_id, created_at"#,
        )
        .bind(owner_id as i64)
        .bind(id as i64)
        .fetch_one(&self.pool)
        .await?;

        Ok(workspace)
    }
}

#[cfg(test)]
mod tests {
    use crate::{models::user::CreateUser, AppState};
    use anyhow::Result;

    #[tokio::test]
    async fn workspace_should_create_and_set_owner() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws = state.create_workspace("test", 0).await?;
        let input = CreateUser::new(&ws.name, "tester01", "tester01@acme.org", "password");

        let user = state.create_user(&input).await?;
        assert_eq!(ws.name, "test");
        assert_eq!(user.ws_id, ws.id);

        let ws = state
            .update_workspace_owner(ws.id as _, user.id as _)
            .await
            .unwrap();
        assert_eq!(ws.owner_id, user.id);
        // tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_find_by_name() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws = state.find_workspace_by_name("acme").await?;
        assert_eq!(ws.unwrap().name, "acme");
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_fetch_all_chat_users() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        // 在 test.sql 中有 5 个用户
        let users = state.fetch_chat_users(1).await?;
        assert_eq!(users.len(), 5);
        Ok(())
    }
}
