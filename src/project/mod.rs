struct Project {
    name: String,
    description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_project_test() {
        let project = Project {
            name: String::from("Project"),
            description: String::from("A sample project"),
        };

        assert_eq!(project.name, "Project");
        assert_eq!(project.description, "A sample project");
    }
}
