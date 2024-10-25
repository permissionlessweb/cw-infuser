
// fn main() {
//     println!("Hello, world!");
// }


// // proposal suite
// pub struct CwInfuserSuite<Chain> {
//     pub infuser: CwInfuser<Chain>,
// }

// impl<Chain: CwEnv> CwInfuserSuite<Chain> {
//     pub fn new(chain: Chain) -> CwInfuserSuite<Chain> {
//         CwInfuserSuite::<Chain> {

//         }
//     }

//     pub fn upload(&self) -> Result<(), CwOrchError> {
//         self.prop_single.upload()?;
//         self.prop_multiple.upload()?;
//         self.prop_condocert.upload()?;
//         self.prop_sudo.upload()?;
//         self.pre_prop_suite.upload()?;
//         Ok(())
//     }
// }