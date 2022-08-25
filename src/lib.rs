use std::{string::FromUtf8Error, str::FromStr};

use k8s_openapi::api::core::v1::Secret;

pub trait SecretExt {
    fn get_u8(&self, field: &str) -> Option<&Vec<u8>>;

    fn get_str(&self, field: &str) -> Result<Option<String>, FromUtf8Error> {
        match self.get_u8(field) {
            Some(value) => Ok(Some(String::from_utf8(value.clone())?)),
            None => Ok(None),
        }
    }

    fn get_from_str<T: FromStr>(&self, field: &str) -> Result<Option<T>, FromStrError<T>> {
        match self.get_str(field)? {
            Some(data) => Ok(Some(T::from_str(&data).map_err(FromStrError::FromStr)?)),
            None => Ok(None),
        }
    }
}

impl SecretExt for Secret {
    fn get_u8(&self, field: &str) -> Option<&Vec<u8>> {
        let data = match self.data {
            Some(ref data) => data,
            None => return None,
        };

        match data.get(field) {
            Some(data) => Some(&data.0),
            None => None,
        }
    }
}

#[derive(thiserror::Error)]
pub enum FromStrError<T: FromStr> {
    FromUtf8(#[from] FromUtf8Error),
    FromStr(T::Err),
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use k8s_openapi::api::core::v1::Secret;
    use kube::{Client, core::ObjectMeta, api::{Api, PostParams, DeleteParams}};
    use ulid::Ulid;

    use super::SecretExt;

    #[tokio::test]
    async fn it_works() {
        let client = Client::try_default().await.unwrap();
        let secrets: Api<Secret> = Api::default_namespaced(client);

        let value = Ulid::new();
        let mut data = BTreeMap::new();
        data.insert("test".to_owned(), value.to_string());

        secrets.create(&PostParams::default(), &Secret {
            metadata: ObjectMeta {
                name: Some("test-secret".to_owned()),
                ..ObjectMeta::default()
            },
            string_data: Some(data),
            ..Secret::default()
        }).await.unwrap();

        let result = secrets.get("test-secret").await.unwrap();
        assert_eq!(Ulid::from_string(&result.get_str("test").unwrap().unwrap()).unwrap(), value);

        secrets.delete("test-secret", &DeleteParams::default()).await.unwrap();
    }
}
