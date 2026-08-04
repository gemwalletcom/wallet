// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import Components
import Foundation
import PerpetualService
import Primitives
import PrimitivesComponents

@MainActor
protocol AssetBalanceActions: AnyObject {
    var balanceService: BalanceService { get }
    var wallet: Wallet { get }
    var isPresentingToastMessage: ToastMessage? { get set }
}

extension AssetBalanceActions {
    func onPinAsset(_ asset: Asset, value: Bool) {
        do {
            try balanceService.setPinned(value, walletId: wallet.id, assetId: asset.id)
            isPresentingToastMessage = .pin(asset.name, pinned: value)
        } catch {
            debugLog("\(Self.self) pin asset error: \(error)")
        }
    }

    func onHideAsset(_ assetId: AssetId) {
        do {
            try balanceService.hideAsset(walletId: wallet.id, assetId: assetId)
        } catch {
            debugLog("\(Self.self) hide asset error: \(error)")
        }
    }
}

@MainActor
protocol AssetEnableActions: AnyObject {
    var assetsEnabler: any AssetsEnabler { get }
    var wallet: Wallet { get }
    var isPresentingToastMessage: ToastMessage? { get set }
}

extension AssetEnableActions {
    func onAddToWallet(_ assetId: AssetId) {
        Task {
            do {
                try await assetsEnabler.enableAssets(wallet: wallet, assetIds: [assetId], enabled: true)
                isPresentingToastMessage = .addedToWallet()
            } catch {
                debugLog("\(Self.self) enable asset error: \(error)")
            }
        }
    }
}

@MainActor
protocol PerpetualPinActions: AnyObject {
    var perpetualService: PerpetualService { get }
    var isPresentingToastMessage: ToastMessage? { get set }
}

extension PerpetualPinActions {
    func onSelectPinPerpetual(_ perpetualData: PerpetualData) {
        let pinned = !perpetualData.metadata.isPinned
        do {
            try perpetualService.setPinned(pinned, perpetualId: perpetualData.perpetual.id)
            isPresentingToastMessage = .pin(perpetualData.perpetual.name, pinned: pinned)
        } catch {
            debugLog("\(Self.self) pin perpetual error: \(error)")
        }
    }
}
