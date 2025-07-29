/// 执行代币转账操作
///
/// # 参数说明
/// * `from` - 源代币账户
/// * `to` - 目标代币账户
/// * `amount` - 转账金额
/// * `mint` - 代币铸造信息
/// * `authority` - 转账授权签名者
/// * `token_program` - 代币程序接口
///
/// # 返回值
/// 返回Result类型，成功时无错误，失败时返回错误信息
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

pub fn transfer_tokens<'info>(
    from: &InterfaceAccount<'info, TokenAccount>,
    to: &InterfaceAccount<'info, TokenAccount>,
    amount: &u64,
    mint: &InterfaceAccount<'info, Mint>,
    authority: &Signer<'info>,
    token_program: &Interface<'info, TokenInterface>,
) -> Result<()> {
    let transfer_accounts_options = TransferChecked {
        from: from.to_account_info(),
        mint: mint.to_account_info(),
        to: to.to_account_info(),
        authority: authority.to_account_info(),
    };

    let cpi_context = CpiContext::new(token_program.to_account_info(), transfer_accounts_options);

    transfer_checked(cpi_context, *amount, mint.decimals)
}
