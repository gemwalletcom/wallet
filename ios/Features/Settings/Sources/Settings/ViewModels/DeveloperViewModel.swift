// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemDeveloperServiceProtocol
import GemstoneServices
import BigInt
import Components
import Foundation
import GemstonePrimitives
import Localization
import Preferences
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@Observable
@MainActor
public final class DeveloperViewModel {
    private let walletId: WalletId
    private let service: any GemDeveloperServiceProtocol
    private let transactionStore: TransactionStore
    private let assetStore: AssetStore
    private let stakeStore: StakeStore
    private let bannerStore: BannerStore
    private let priceStore: PriceStore

    public var isPresentingToastMessage: ToastMessage?
    public private(set) var deviceId: String = .empty
    public private(set) var deviceToken: String = .empty

    public init(
        walletId: WalletId,
        service: any GemDeveloperServiceProtocol,
        transactionStore: TransactionStore,
        assetStore: AssetStore,
        stakeStore: StakeStore,
        bannerStore: BannerStore,
        priceStore: PriceStore,
    ) {
        self.walletId = walletId
        self.service = service
        self.transactionStore = transactionStore
        self.assetStore = assetStore
        self.stakeStore = stakeStore
        self.bannerStore = bannerStore
        self.priceStore = priceStore
    }

    var title: String {
        Localized.Settings.developer
    }

    func load() async {
        deviceId = (try? await service.deviceId()) ?? .empty
        deviceToken = (try? await service.pushToken()) ?? .empty
    }

    func reset() {
        do {
            try clearDocuments()
            try service.clearPreferences()
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
            try transactionStore.clear()
        }
    }

    func clearPendingTransactions() {
        Task {
            await performAction {
                try await service.clearPendingTransactions()
            }
        }
    }

    func clearTransactionsTimestamp() {
        performAction {
            try service.resetTransactionsTimestamp(walletId: walletId.id)
        }
    }

    func clearWalletPreferences() {
        performAction {
            try service.deleteWalletPreferences(walletId: walletId.id)
        }
    }

    func clearAssets() {
        performAction {
            try assetStore.clearTokens()
        }
    }

    func clearDelegations() {
        performAction {
            try stakeStore.clearDelegations()
        }
    }

    func clearValidators() {
        performAction {
            try stakeStore.clearValidators()
        }
    }

    func clearBanners() {
        performAction {
            _ = try bannerStore.clear()
        }
    }

    func activateAllCancelledBanners() {
        performAction {
            _ = try bannerStore.updateStates(from: .cancelled, to: .active)
        }
    }

    func clearPrices() {
        performAction {
            _ = try priceStore.clear()
        }
    }

    func clearPerpetuals() {
        Task {
            await performAction {
                try await service.clearPerpetualMarkets()
            }
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
        try? transactionStore.addTransactions(walletId: walletId, transactions: transactions)
    }

    func deeplink(deeplink: DeepLink) {
        Task { @MainActor in
            await UIApplication.shared.open(service.deeplinkUrl(deeplink: deeplink.map()).asURL!, options: [:])
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

    private func performAction(_ action: () async throws -> Void) async {
        do {
            try await action()
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
