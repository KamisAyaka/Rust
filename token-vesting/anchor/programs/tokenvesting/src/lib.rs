// 允许一些 clippy 警告（Rust 代码质量检查工具）
#![allow(clippy::result_large_err)]
// 允许使用意外的 cfg 条件编译属性
#![allow(unexpected_cfgs)]

// 引入 Anchor 框架的核心功能
use anchor_lang::prelude::*;
// 引入 Anchor 提供的关联代币和代币接口功能
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};

// 声明程序的 ID（唯一标识符），这是部署时生成的
declare_id!("AfJ7jgnc2VQ2tzTrNzVzCrq6VtHi9DhzYFuUUFmh49jF");

// 主程序模块，包含所有业务逻辑
#[program]
pub mod tokenvesting {
    use super::*;

    // 创建托管账户的函数
    // 接受上下文和公司名称作为参数
    pub fn create_vesting_account(
        ctx: Context<CreateVestingAccount>, // 上下文包含所有需要的账户
        company_name: String,               // 公司名称
    ) -> Result<()> {
        // 初始化托管账户，填充所有必要字段
        *ctx.accounts.vesting_account = VestingAccount {
            owner: ctx.accounts.signer.key(), // 设置所有者为交易签名者
            mint: ctx.accounts.mint.key(),    // 设置代币类型
            treasury_token_account: ctx.accounts.treasury_token_account.key(), // 设置金库账户
            company_name,                     // 公司名称
            treasury_bump: ctx.bumps.treasury_token_account, // 用于 PDA 的随机数
            bump: ctx.bumps.vesting_account,  // 用于 PDA 的随机数
        };
        Ok(()) // 返回成功结果
    }

    // 创建员工账户的函数
    pub fn create_employee_account(
        ctx: Context<CreateEmployeeAccount>, // 上下文包含所有需要的账户
        start_time: i64,                     // 开始时间（Unix 时间戳）
        end_time: i64,                       // 结束时间（Unix 时间戳）
        total_amount: u64,                   // 总代币数量
        cliff_time: i64,                     // 悬崖期结束时间（在此时间之前无法提取）
    ) -> Result<()> {
        // 初始化员工账户，填充所有必要字段
        *ctx.accounts.employee_account = EmployeeAccount {
            beneficiary: ctx.accounts.beneficiary.key(), // 受益人公钥
            start_time,                                  // 开始时间
            end_time,                                    // 结束时间
            total_amount,                                // 总金额
            total_withdrawn: 0,                          // 已提取金额初始化为 0
            cliff_time,                                  // 悬崖期时间
            vesting_account: ctx.accounts.vesting_account.key(), // 所属托管账户
            bump: ctx.bumps.employee_account,            // PDA 随机数
        };
        Ok(()) // 返回成功结果
    }

    // 员工提取代币的函数
    pub fn claim_tokens(ctx: Context<ClaimTokens>, _company_name: String) -> Result<()> {
        // 获取员工账户的可变引用
        let employee_account = &mut ctx.accounts.employee_account;
        // 获取当前的区块链时间戳
        let now = Clock::get()?.unix_timestamp;

        // 如果当前时间早于悬崖期结束时间，则无法提取
        if now < employee_account.cliff_time {
            return Err(ErrorCode::CliamNotAvailableYet.into());
        }

        // 计算从开始时间到现在的经过时间
        let time_since_start: i64 = now.saturating_sub(employee_account.start_time);
        // 计算总的归属期时间
        let total_vesting_time = employee_account
            .end_time
            .saturating_sub(employee_account.start_time);

        // 如果总的归属期时间为0，说明参数错误
        if total_vesting_time == 0 {
            return Err(ErrorCode::InvalidVestPeriod.into());
        }

        // 计算当前应该归属的代币数量
        let vested_amount: u64 = if now >= employee_account.end_time {
            // 如果已经过了归属期结束时间，全部代币都已归属
            employee_account.total_amount
        } else {
            // 否则按时间比例计算归属数量
            match employee_account
                .total_amount
                .checked_mul(time_since_start as u64)
            {
                Some(product) => product / total_vesting_time as u64,
                None => return Err(ErrorCode::CalculationOverflow.into()), // 检查溢出
            }
        };

        // 计算本次可提取的代币数量（已归属但未提取的部分）
        let claimable_amount = vested_amount.saturating_sub(employee_account.total_withdrawn);
        // 如果没有可提取的代币，则返回错误
        if claimable_amount == 0 {
            return Err(ErrorCode::NothingToClaim.into());
        }

        // 准备代币转账的 CPI（跨程序调用）账户
        let transfer_cpi_accounts = TransferChecked {
            from: ctx.accounts.treasury_token_account.to_account_info(), // 从金库账户转出
            mint: ctx.accounts.mint.to_account_info(),                   // 代币铸造账户
            to: ctx.accounts.employee_token_account.to_account_info(),   // 转入员工账户
            authority: ctx.accounts.treasury_token_account.to_account_info(), // 授权账户
        };

        // 获取代币程序账户信息
        let cpi_program = ctx.accounts.token_program.to_account_info();
        // 构造签名种子，用于证明 PDA 的有效性
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"treasury_token_account",
            ctx.accounts.vesting_account.company_name.as_ref(),
            &[ctx.accounts.vesting_account.treasury_bump],
        ]];
        // 创建 CPI 上下文并添加签名
        let cpi_context =
            CpiContext::new(cpi_program, transfer_cpi_accounts).with_signer(signer_seeds);
        // 获取代币小数位数
        let decimals = ctx.accounts.mint.decimals;
        // 执行代币转账
        token_interface::transfer_checked(cpi_context, claimable_amount as u64, decimals)?;

        // 更新已提取金额
        employee_account.total_withdrawn += claimable_amount;
        Ok(()) // 返回成功结果
    }
}

// 定义创建员工账户所需的账户结构
#[derive(Accounts)]
pub struct CreateEmployeeAccount<'info> {
    // 所有者账户，必须是交易签名者且可变
    #[account(mut)]
    pub owner: Signer<'info>,

    // 受益人账户，系统账户类型
    pub beneficiary: SystemAccount<'info>,

    // 托管账户，必须由所有者拥有
    #[account(
        has_one = owner,
    )]
    pub vesting_account: Account<'info, VestingAccount>,

    // 员工账户，初始化时创建
    #[account(
        init, // 初始化账户
        payer = owner, // owner 支付租金
        space = 8 + EmployeeAccount::INIT_SPACE, // 分配空间
        seeds = [b"employee_vesting", beneficiary.key().as_ref(), vesting_account.key().as_ref()], // PDA 种子
        bump, // 自动查找 bump
    )]
    pub employee_account: Account<'info, EmployeeAccount>,

    // 系统程序，用于创建账户
    pub system_program: Program<'info, System>,
}

// 定义提取代币所需的账户结构
#[derive(Accounts)]
#[instruction(company_name: String)]
pub struct ClaimTokens<'info> {
    // 受益人账户，必须是交易签名者且可变
    #[account(mut)]
    pub beneficiary: Signer<'info>,

    // 员工账户，必须与受益人和托管账户匹配
    #[account(
        mut, // 可变
        seeds = [b"employee_vesting", beneficiary.key().as_ref(), vesting_account.key().as_ref()], // PDA 种子
        bump = employee_account.bump, // 使用存储的 bump
        has_one = beneficiary, // 必须属于 beneficiary
        has_one = vesting_account, // 必须属于 vesting_account
    )]
    pub employee_account: Account<'info, EmployeeAccount>,

    // 托管账户，必须与金库账户和代币类型匹配
    #[account(
        mut, // 可变
        seeds = [company_name.as_bytes()], // PDA 种子
        bump = vesting_account.bump, // 使用存储的 bump
        has_one = treasury_token_account, // 必须拥有 treasury_token_account
        has_one = mint // 必须拥有 mint
    )]
    pub vesting_account: Account<'info, VestingAccount>,

    // 代币铸造账户
    pub mint: InterfaceAccount<'info, Mint>,

    // 金库代币账户，可变
    #[account(mut)]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,

    // 员工代币账户，如果不存在则创建
    #[account(
        init_if_needed, // 如果不存在则初始化
        payer = beneficiary, // beneficiary 支付租金
        associated_token::mint = mint, // 关联的代币类型
        associated_token::authority = beneficiary, // 关联的所有者
        associated_token::token_program = token_program, // 使用的代币程序
    )]
    pub employee_token_account: InterfaceAccount<'info, TokenAccount>,

    // 系统程序
    pub system_program: Program<'info, System>,
    // 代币程序
    pub token_program: Interface<'info, TokenInterface>,
    // 关联代币程序
    pub associated_token_program: Program<'info, AssociatedToken>,
}

// 员工账户结构体定义
#[account]
#[derive(InitSpace)]
pub struct EmployeeAccount {
    pub beneficiary: Pubkey,     // 受益人公钥
    pub start_time: i64,         // 开始时间
    pub end_time: i64,           // 结束时间
    pub cliff_time: i64,         // 悬崖期结束时间
    pub vesting_account: Pubkey, // 所属托管账户
    pub total_amount: u64,       // 总金额
    pub total_withdrawn: u64,    // 已提取金额
    pub bump: u8,                // PDA 的 bump 值
}

// 定义创建托管账户所需的账户结构
#[derive(Accounts)]
#[instruction(company_name: String)]
pub struct CreateVestingAccount<'info> {
    // 签名者账户，必须是交易签名者且可变
    #[account(mut)]
    pub signer: Signer<'info>,

    // 托管账户，初始化时创建
    #[account(
        init, // 初始化
        payer = signer, // signer 支付租金
        space = 8 + VestingAccount::INIT_SPACE, // 分配空间
        seeds = [company_name.as_bytes()], // PDA 种子
        bump // 自动查找 bump
    )]
    pub vesting_account: Account<'info, VestingAccount>,

    // 代币铸造账户
    pub mint: InterfaceAccount<'info, Mint>,

    // 金库代币账户，初始化时创建
    #[account(
        init, // 初始化
        token::mint = mint, // 关联的代币类型
        token::authority = treasury_token_account, // 授权账户
        payer = signer, // signer 支付租金
        seeds = [b"treasury_token_account", company_name.as_bytes()], // PDA 种子
        bump // 自动查找 bump
    )]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,

    // 系统程序
    pub system_program: Program<'info, System>,
    // 代币程序
    pub token_program: Interface<'info, TokenInterface>,
}

// 托管账户结构体定义
#[account]
#[derive(InitSpace)]
pub struct VestingAccount {
    pub owner: Pubkey,                  // 所有者公钥
    pub mint: Pubkey,                   // 代币类型公钥
    pub treasury_token_account: Pubkey, // 金库账户公钥
    #[max_len(32)] // 公司名称最大长度为32字节
    pub company_name: String, // 公司名称
    pub treasury_bump: u8,              // 金库账户的 bump 值
    pub bump: u8,                       // 托管账户的 bump 值
}

// 自定义错误代码枚举
#[error_code]
pub enum ErrorCode {
    #[msg("cliam not available yet")] // 提取时间未到
    CliamNotAvailableYet,
    #[msg("invalid vest period")] // 无效的归属期
    InvalidVestPeriod,
    #[msg("calculation overflow")] // 计算溢出
    CalculationOverflow,
    #[msg("Nothing to cliam")] // 没有可提取的代币
    NothingToClaim,
}
