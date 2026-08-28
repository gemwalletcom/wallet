// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemBalanceServiceProtocol
import GemstonePrimitives
import protocol Gemstone.GemPerpetualServiceProtocol
import GemstoneServices
import Components
import Foundation
import Primitives
import PrimitivesComponents

@MainActor
protocol AssetActions: AnyObject {
    var balanceService: any GemBalanceServiceProtocol { get }
    var wallet: Wallet { get }
    var isPresentingToastMessage: ToastMessage? { get set }
}

extension AssetActions {
    func onPinAsset(_ asset: Asset, value: Bool) {
        Task {
            do {
                try await balanceService.pinAsset(wallet: wallet, assetId: asset.id, pinned: value)
                isPresentingToastMessage = .pin(asset.name, pinned: value)
            } catch {
                debugLog("\(Self.self) pin asset error: \(error)")
            }
        }
    }

    func onHideAsset(_ assetId: AssetId) {
        Task {
            do {
                try await balanceService.enableAssets(wallet: wallet, assetIds: [assetId], enabled: false)
            } catch {
                debugLog("\(Self.self) hide asset error: \(error)")
            }
        }
    }

    func onAddToWallet(_ assetId: AssetId) {
        Task {
            do {
                try await balanceService.enableAssets(wallet: wallet, assetIds: [assetId], enabled: true)
                isPresentingToastMessage = .addedToWallet()
            } catch {
                debugLog("\(Self.self) enable asset error: \(error)")
            }
        }
    }
}

@MainActor
protocol PerpetualPinActions: AnyObject {
    var perpetualService: any GemPerpetualServiceProtocol { get }
    var isPresentingToastMessage: ToastMessage? { get set }
}

extension PerpetualPinActions {
    func onSelectPinPerpetual(_ perpetualData: PerpetualData) {
        let pinned = !perpetualData.metadata.isPinned
        Task {
            do {
                try await perpetualService.setPinned(pinned, perpetualId: perpetualData.perpetual.id)
                isPresentingToastMessage = .pin(perpetualData.perpetual.name, pinned: pinned)
            } catch {
                debugLog("\(Self.self) pin perpetual error: \(error)")
            }
        }
    }
}
