struct Project {
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_project_test() {
        let project = Project {
            name: String::from("Project"),
        };

        assert_eq!(project.name, "Project")
    }
}
