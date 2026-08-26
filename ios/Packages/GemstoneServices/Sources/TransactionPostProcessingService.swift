// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemBalanceServiceProtocol
import protocol Gemstone.GemNftServiceProtocol
import Foundation
import protocol Gemstone.GemStakeServiceProtocol
import Primitives
import Store

public struct TransactionPostProcessingService: Sendable {
    private let transactionStore: TransactionStore
    private let balanceService: any GemBalanceServiceProtocol
    private let stakeService: any GemStakeServiceProtocol
    private let nftService: any GemNftServiceProtocol

    public init(
        transactionStore: TransactionStore,
        balanceService: any GemBalanceServiceProtocol,
        stakeService: any GemStakeServiceProtocol,
        nftService: any GemNftServiceProtocol,
    ) {
        self.transactionStore = transactionStore
        self.balanceService = balanceService
        self.stakeService = stakeService
        self.nftService = nftService
    }

    func process(wallet: Wallet, transaction: Transaction) async throws {
        await updateBalances(wallet: wallet, transaction: transaction)

        switch transaction.type {
        case .stakeDelegate, .stakeUndelegate, .stakeRewards, .stakeRedelegate, .stakeWithdraw:
            for assetIdentifier in transaction.assetIds {
                Task {
                    try await stakeService.sync(
                        walletId: wallet.id.id,
                        chain: assetIdentifier.chain.rawValue,
                        address: transaction.from,
                    )
                }
            }
        case .earnDeposit, .earnWithdraw:
            for assetIdentifier in transaction.assetIds {
                Task {
                    try await stakeService.syncEarn(
                        walletId: wallet.id.id,
                        assetId: assetIdentifier.identifier,
                        address: transaction.from,
                    )
                }
            }
        case .transferNFT:
            _ = try await nftService.sync(walletId: wallet.id.id)
        default:
            break
        }
    }

    func updateBalances(wallet: Wallet, transaction: Transaction) async {
        do {
            try await balanceService.update(walletId: wallet.id.id, assetIds: transaction.associatedAssetIds.ids)
        } catch {
            debugLog("update balance error: \(error)")
        }
    }
}
