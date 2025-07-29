//! voteing 程序模块
//! 实现投票系统的链上逻辑，包含投票初始化、候选人管理和投票计数功能
//!
//! 主要功能：
//! - 创建投票活动
//! - 添加候选人
//! - 处理投票操作
//!
//! 主要数据结构：
//! - Poll: 投票活动信息
//! - Candidate: 候选人信息

#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;

// 程序ID声明，与Anchor.toml配置保持一致
declare_id!("7qndvv9MS9WWNRZctz3gVVYaaDhv3VsT4QhgAAsaA573");

#[program]
pub mod voteing {
    use super::*;

    /// 初始化投票活动
    ///
    /// 创建一个新的投票活动账户，设置基础信息
    ///
    /// # 参数说明
    /// - poll_id: 投票ID（唯一标识）
    /// - description: 投票描述文本
    /// - poll_start: 投票开始时间（Unix时间戳）
    /// - poll_end: 投票结束时间（Unix时间戳）
    pub fn initialize_poll(
        ctx: Context<InitializePoll>,
        poll_id: u64,
        description: String,
        poll_start: u64,
        poll_end: u64,
    ) -> Result<()> {
        let poll = &mut ctx.accounts.poll;
        poll.poll_id = poll_id;
        poll.description = description;
        poll.poll_start = poll_start;
        poll.poll_end = poll_end;
        poll.candidate_amount = 0;
        Ok(())
    }

    /// 初始化候选人信息
    ///
    /// 为指定投票活动创建候选人账户
    ///
    /// # 参数说明
    /// - candidate_name: 候选人名称
    /// - poll_id: 关联的投票ID（通过上下文获取）
    pub fn initialize_candidate(
        ctx: Context<InitializeCandidate>,
        candidate_name: String,
        poll_id: u64,
    ) -> Result<()> {
        let candidate = &mut ctx.accounts.candidate;
        candidate.poll_id = poll_id;
        candidate.candidate_name = candidate_name;
        candidate.candidate_votes = 0;
        Ok(())
    }
    /// 处理投票操作
    ///
    /// 增加指定候选人的得票数
    ///
    /// # 参数说明
    /// - _candidate_name: 被投票的候选人名称（通过账户验证）
    /// - _poll_id: 投票ID（通过账户验证）
    pub fn vote(ctx: Context<Vote>, _candidate_name: String, _poll_id: u64) -> Result<()> {
        let candidate = &mut ctx.accounts.candidate;

        // 增加候选人得票数
        candidate.candidate_votes += 1;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(poll_id: u64)]
pub struct InitializePoll<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        space = 8 + Poll::INIT_SPACE,
        seeds = [poll_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub poll: Account<'info, Poll>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(candidate_name:String,poll_id: u64)]
pub struct InitializeCandidate<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [poll_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub poll: Account<'info, Poll>,
    #[account(
        init,
        payer = authority,
        space = 8 + Candidate::INIT_SPACE,
        seeds = [poll_id.to_le_bytes().as_ref(), candidate_name.as_bytes()],
        bump,
    )]
    pub candidate: Account<'info, Candidate>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(candidate_name:String,poll_id: u64)]
pub struct Vote<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        seeds = [poll_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub poll: Account<'info, Poll>,
    #[account(
        mut,
        seeds = [poll_id.to_le_bytes().as_ref(), candidate_name.as_bytes()],
        bump,
    )]
    pub candidate: Account<'info, Candidate>,
    pub system_program: Program<'info, System>,
}

/// 候选人账户结构
///
/// 用于存储候选人基本信息和得票数
#[account]
#[derive(InitSpace)]
pub struct Candidate {
    pub poll_id: u64,
    #[max_len(32)]
    pub candidate_name: String, // 候选人名称，最大长度32字节
    pub candidate_votes: u64, // 得票数统计
}

/// 投票活动账户结构
///
/// 用于存储投票活动的基础信息
#[account]
#[derive(InitSpace)]
pub struct Poll {
    pub poll_id: u64, // 投票活动唯一ID
    #[max_len(280)]
    pub description: String, // 投票描述文本
    pub poll_start: u64, // 投票开始时间（Unix时间戳）
    pub poll_end: u64, // 投票结束时间（Unix时间戳）
    pub candidate_amount: u64, // 候选人总数统计
}
