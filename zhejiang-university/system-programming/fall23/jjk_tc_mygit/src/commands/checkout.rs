use crate::commands::CommandContext;
use crate::database::object::Object;
use crate::database::tree::TreeEntry;
use crate::database::tree_diff::TreeDiff;
use crate::database::{Database, ParsedObject};
use crate::refs::Ref;
use crate::repository::Repository;
use crate::revision::Revision;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;

const DETACHED_HEAD_MESSAGE: &str =
    "You are in 'detached HEAD' state. You can look around, make experimental 
changes and commit them, and you can discard any commits you make in this
 state without impacting any branches by performing another checkout.

If you want to create a new branch to retain commits you create, you may
do so (now or later) by using the branch command. Example:

  rug branch <new-branch-name>
";

pub struct Checkout<'a, I, O, E>
where
    I: Read,
    O: Write,
    E: Write,
{
    repo: Repository,
    ctx: CommandContext<'a, I, O, E>,
}

impl<'a, I, O, E> Checkout<'a, I, O, E>
where
    I: Read,
    O: Write,
    E: Write,
{
    pub fn new(ctx: CommandContext<'a, I, O, E>) -> Checkout<'a, I, O, E> {
        let working_dir = &ctx.dir;
        let root_path = working_dir.as_path();
        let repo = Repository::new(&root_path);

        Checkout { repo, ctx }
    }

    fn print_head_position(&mut self, message: &str, oid: &str) -> Result<(), String> {
        let commit = match self.repo.database.load(oid) {
            ParsedObject::Commit(commit) => commit,
            _ => panic!("oid not a commit"),
        };
        let oid = commit.get_oid();
        let short = Database::short_oid(&oid);

        println!(
            "{}",
            format!("{} {} {}", message, short, commit.title_line())
        );
        Ok(())
    }

    fn print_previous_head(
        &mut self,
        current_ref: &Ref,
        current_oid: &str,
        target_oid: &str,
    ) -> Result<(), String> {
        if current_ref.is_head() && current_oid != target_oid {
            return self.print_head_position("Previous HEAD position was", current_oid);
        }
        Ok(())
    }

    fn print_detachment_notice(
        &mut self,
        current_ref: &Ref,
        target: &str,
        new_ref: &Ref,
    ) -> Result<(), String> {
        if new_ref.is_head() && !current_ref.is_head() {
            println!(
                "{}

{}
",
                format!("Note: checking out '{}'.", target),
                DETACHED_HEAD_MESSAGE
            );
        }
        Ok(())
    }

    fn print_new_head(
        &mut self,
        current_ref: &Ref,
        new_ref: &Ref,
        target: &str,
        target_oid: &str,
    ) -> Result<(), String> {
        if new_ref.is_head() {
            self.print_head_position("HEAD is now at", target_oid)?;
        } else if new_ref == current_ref {
            eprintln!("{}", format!("Already on {}", target));
        } else {
            eprintln!("{}", format!("Switched to branch {}", target));
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), String> {
        let options = self.ctx.options.as_ref().unwrap().clone();
        let args: Vec<_> = if let Some(args) = options.values_of("args") {
            args.collect()
        } else {
            vec![]
        };
        let target = args.get(0).expect("no target provided");

        self.repo
            .index
            .load_for_update()
            .map_err(|e| e.to_string())?;

        let current_ref = self.repo.refs.current_ref("HEAD");
        let current_oid = self
            .repo
            .refs
            .read_oid(&current_ref)
            .unwrap_or_else(|| panic!("failed to read ref: {:?}", current_ref));

        let mut revision = Revision::new(&mut self.repo, target);
        let target_oid = match revision.resolve() {
            Ok(oid) => oid,
            Err(errors) => {
                let mut v = vec![];
                for error in errors {
                    v.push(format!("error: {}", error.message));
                    for h in error.hint {
                        v.push(format!("hint: {}", h));
                    }
                }

                v.push("\n".to_string());

                return Err(v.join("\n"));
            }
        };

        let tree_diff = self.tree_diff(&current_oid, &target_oid);
        let mut migration = self.repo.migration(tree_diff);
        migration.apply_changes()?;

        self.repo.index.write_updates().map_err(|e| e.to_string())?;
        self.repo
            .refs
            .set_head(&target, &target_oid)
            .map_err(|e| e.to_string())?;

        let new_ref = self.repo.refs.current_ref("HEAD");
        self.print_previous_head(&current_ref, &current_oid, &target_oid)?;
        self.print_detachment_notice(&current_ref, &target, &new_ref)?;
        self.print_new_head(&current_ref, &new_ref, &target, &target_oid)?;

        Ok(())
    }

    fn tree_diff(
        &mut self,
        a: &str,
        b: &str,
    ) -> HashMap<PathBuf, (Option<TreeEntry>, Option<TreeEntry>)> {
        let mut td = TreeDiff::new(&mut self.repo.database);
        td.compare_oids(
            Some(a.to_string()),
            Some(b.to_string()),
            std::path::Path::new(""),
        );
        td.changes
    }
}