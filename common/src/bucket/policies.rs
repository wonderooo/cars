pub fn public_bucket_policy<'a>(bucket: impl Into<&'a str>) -> String {
    format!(
        r#"{{
            "Version":"2012-10-17",
            "Statement":[{{
                "Sid":"PublicReadGetObject",
                "Effect":"Allow",
                "Principal":"*",
                "Action":["s3:GetObject"],
                "Resource":["arn:aws:s3:::{bucket}/*"]
            }}]
        }}"#,
        bucket = bucket.into()
    )
}
