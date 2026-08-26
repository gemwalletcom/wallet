// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemStakeServiceProtocol
import Primitives
import Store

public struct TransactionPostProcessingService: Sendable {
    private let transactionStore: TransactionStore
    private let balanceUpdater: any BalanceUpdater
    private let stakeService: any GemStakeServiceProtocol
    private let nftService: NFTService

    public init(
        transactionStore: TransactionStore,
        balanceUpdater: any BalanceUpdater,
        stakeService: any GemStakeServiceProtocol,
        nftService: NFTService,
    ) {
        self.transactionStore = transactionStore
        self.balanceUpdater = balanceUpdater
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
            try await nftService.updateAssets(wallet: wallet)
        default:
            break
        }
    }

    func updateBalances(wallet: Wallet, transaction: Transaction) async {
        await balanceUpdater.updateBalance(
            for: wallet,
            assetIds: transaction.associatedAssetIds,
        )
    }
}
