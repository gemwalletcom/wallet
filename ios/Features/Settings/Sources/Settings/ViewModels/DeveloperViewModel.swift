// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import BannerService
import BigInt
import Components
import Foundation
import Localization
import PerpetualService
import Preferences
import PriceService
import Primitives
import PrimitivesComponents
import StakeService
import SwiftUI
import TransactionsService

@Observable
@MainActor
public final class DeveloperViewModel {
    private let walletId: WalletId
    private let transactionsService: TransactionsService
    private let assetService: AssetsService
    private let stakeService: StakeService
    private let bannerService: BannerService
    private let priceService: PriceService
    private let perpetualService: PerpetualService

    public var isPresentingToastMessage: ToastMessage?

    public init(
        walletId: WalletId,
        transactionsService: TransactionsService,
        assetService: AssetsService,
        stakeService: StakeService,
        bannerService: BannerService,
        priceService: PriceService,
        perpetualService: PerpetualService,
    ) {
        self.walletId = walletId
        self.transactionsService = transactionsService
        self.assetService = assetService
        self.stakeService = stakeService
        self.bannerService = bannerService
        self.priceService = priceService
        self.perpetualService = perpetualService
    }

    var title: String {
        Localized.Settings.developer
    }

    var deviceId: String {
        (try? SecurePreferences.standard.getDeviceId()) ?? .empty
    }

    var deviceToken: String {
        (try? SecurePreferences().get(key: .deviceToken)) ?? .empty
    }

    func reset() {
        do {
            try clearDocuments()
            Preferences.standard.clear()
            try SecurePreferences.standard.clear()
            fatalError()
        } catch {
            debugLog("reset error \(error)")
        }
    }

    func clearCache() {
        performAction {
            URLCache.shared.removeAllCachedResponses()
        }
    }

    func clearTransactions() {
        performAction {
            try transactionsService.transactionStore.clear()
        }
    }

    func clearPendingTransactions() {
        performAction {
            let transactionIds = try transactionsService.transactionStore.getTransactions(state: .pending).map(\.id.identifier)
            _ = try transactionsService.transactionStore.deleteTransactionId(ids: transactionIds)
        }
    }

    func clearTransactionsTimestamp() {
        performAction {
            let preferences = WalletPreferences(walletId: walletId)
            preferences.transactionsTimestamp = 0
            preferences.completeInitialLoadTransactions = false
        }
    }

    func clearWalletPreferences() {
        performAction {
            WalletPreferences(walletId: walletId).clear()
        }
    }

    func clearAssets() {
        performAction {
            try assetService.assetStore.clearTokens()
        }
    }

    func clearDelegations() {
        performAction {
            try stakeService.clearDelegations()
        }
    }

    func clearValidators() {
        performAction {
            try stakeService.clearValidators()
        }
    }

    func clearBanners() {
        performAction {
            try bannerService.clearBanners()
        }
    }

    func activateAllCancelledBanners() {
        performAction {
            try bannerService.activateAllCancelledBanners()
        }
    }

    func clearPrices() {
        performAction {
            try priceService.clear()
        }
    }

    func clearPerpetuals() {
        performAction {
            try perpetualService.clear()
            Preferences.standard.perpetualMarketsUpdatedAt = .none
        }
    }

    func addTransactions() {
        let solAddress = "7nVDzZUjrBA3gHs3gNcHidhmR96CH7KpKsU8pyBZGHUr"
        let ethAddress = "0xf1158986419F6058231b0Dbd7A78Ff0674ebBc50"
        let btcAddress = "bc1q4jwwsy7txnzsr7w53j4wnrg6rrnmj86a47e2t9"
        let trxAddress = "TAw8sw21A3pGDCtHGuB55BGDqLVHQTYwAC"
        let data: [(direction: TransactionDirection, from: String, to: String, assetId: AssetId, transactionType: TransactionType, value: BigInt, metadata: AnyCodableValue?, createdAt: Date)] = [
            (.incoming, solAddress, "", AssetId(chain: .solana), .transfer, BigInt(111_111_111), .none, createdAt: Date().addingTimeInterval(-1)),
            (.outgoing, "", solAddress, AssetId(chain: .solana), .transfer, BigInt(3_311_111_111), .none, createdAt: Date().addingTimeInterval(-2)),
            (
                .selfTransfer,
                "",
                "",
                AssetId(chain: .sui),
                .swap,
                BigInt(76_767_623_311_111_111),
                .encode(TransactionSwapMetadata(
                    fromAsset: AssetId(chain: .sui),
                    fromValue: BigInt(2_767_611_111).description,
                    toAsset: AssetId(chain: .solana),
                    toValue: BigInt(812_312_312).description,
                    provider: .none,
                )),
                createdAt: Date().addingTimeInterval(-122_223),
            ),
            (
                .incoming,
                trxAddress,
                "",
                AssetId(chain: .tron),
                .transfer,
                BigInt(912_312_312),
                .none,
                createdAt: Date().addingTimeInterval(-122_224),
            ),
            (
                .outgoing,
                "",
                ethAddress,
                AssetId(chain: .ethereum),
                .transfer,
                BigInt(76_767_623_311_111_111),
                .none,
                createdAt: Date().addingTimeInterval(-1_344_411),
            ),
            (
                .incoming,
                btcAddress,
                "",
                AssetId(chain: .bitcoin),
                .transfer,
                BigInt(621_111_111),
                .none,
                createdAt: Date().addingTimeInterval(-100),
            ),
            (
                .incoming,
                btcAddress,
                "",
                AssetId(chain: .bitcoin),
                .transfer,
                BigInt(46_161_111),
                .none,
                createdAt: Date().addingTimeInterval(-10000),
            ),
            (
                .incoming,
                btcAddress,
                "",
                AssetId(chain: .bitcoin),
                .transfer,
                BigInt(72_312_312),
                .none,
                createdAt: Date().addingTimeInterval(-1_344_401),
            ),
            (
                .selfTransfer,
                "",
                "",
                AssetId(chain: .ethereum),
                .swap,
                BigInt(76_767_623_311_111_111),
                .encode(TransactionSwapMetadata(
                    fromAsset: AssetId(chain: .ethereum),
                    fromValue: BigInt(276_767_623_311_111_111).description,
                    toAsset: AssetId(chain: .bitcoin),
                    toValue: BigInt(32_312_312).description,
                    provider: .none,
                )),
                createdAt: Date().addingTimeInterval(-1_344_411),
            ),
            (
                .incoming,
                "",
                "",
                AssetId(chain: .smartChain),
                .stakeRewards,
                BigInt(464_222_222_272_312_312),
                .none,
                createdAt: Date().addingTimeInterval(-1_444_401),
            ),
            (
                .incoming,
                "",
                "NodeReal",
                AssetId(chain: .smartChain),
                .stakeDelegate,
                BigInt("54213322222272312312"),
                .none,
                createdAt: Date().addingTimeInterval(-1_464_401),
            ),
        ]

        let transactions = data.enumerated().map { index, element in
            Transaction(
                id: TransactionId(chain: element.assetId.chain, hash: "\(index)"),
                assetId: element.assetId,
                from: element.from,
                to: element.to,
                contract: .none,
                type: element.transactionType,
                state: .confirmed,
                blockNumber: .zero,
                sequence: .zero,
                fee: .zero,
                feeAssetId: element.assetId,
                value: element.value.description,
                memo: .none,
                direction: element.direction,
                utxoInputs: [],
                utxoOutputs: [],
                metadata: element.metadata,
                createdAt: element.createdAt,
            )
        }
        try? transactionsService.transactionStore.addTransactions(walletId: walletId, transactions: transactions)
    }

    // preferences

    func clearAssetsVersion() {
        performAction {
            Preferences.standard.swapAssetsVersion = 0
        }
    }

    func deeplink(deeplink: DeepLink) {
        Task { @MainActor in
            await UIApplication.shared.open(deeplink.localUrl, options: [:])
        }
    }
}

// MARK: - Private

extension DeveloperViewModel {
    private func showSuccess() {
        isPresentingToastMessage = .success(Localized.Transaction.Status.confirmed)
    }

    private func performAction(_ action: () throws -> Void) {
        do {
            try action()
            showSuccess()
        } catch {
            debugLog("Developer action error: \(error)")
        }
    }

    private func clearDocuments() throws {
        let documentsUrl = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
        let fileURLs = try FileManager.default.contentsOfDirectory(at: documentsUrl, includingPropertiesForKeys: nil, options: .skipsHiddenFiles)
        for fileURL in fileURLs {
            try FileManager.default.removeItem(at: fileURL)
        }
    }
}
