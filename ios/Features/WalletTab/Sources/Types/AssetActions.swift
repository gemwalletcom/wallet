// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import Components
import Foundation
import PerpetualService
import Primitives
import PrimitivesComponents

@MainActor
protocol AssetActions: AnyObject {
    var assetsEnabler: any AssetsEnabler { get }
    var wallet: Wallet { get }
    var isPresentingToastMessage: ToastMessage? { get set }
}

extension AssetActions {
    func onPinAsset(_ asset: Asset, value: Bool) {
        Task {
            do {
                try await assetsEnabler.pinAsset(wallet: wallet, assetId: asset.id, pinned: value)
                isPresentingToastMessage = .pin(asset.name, pinned: value)
            } catch {
                debugLog("\(Self.self) pin asset error: \(error)")
            }
        }
    }

    func onHideAsset(_ assetId: AssetId) {
        Task {
            do {
                try await assetsEnabler.enableAssets(wallet: wallet, assetIds: [assetId], enabled: false)
            } catch {
                debugLog("\(Self.self) hide asset error: \(error)")
            }
        }
    }

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
