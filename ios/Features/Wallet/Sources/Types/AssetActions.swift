// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import protocol Gemstone.GemAssetSelectionServiceProtocol
import enum Gemstone.GemServiceError
import struct Gemstone.GemToast
import GemstonePrimitives
import Primitives
import PrimitivesComponents

@MainActor
protocol AssetActions: AnyObject {
    var isPresentingToastMessage: ToastMessage? { get set }
    func setAssetPinned(_ asset: Asset, pinned: Bool) async throws -> GemToast
    func setAssetsEnabled(_ assetIds: [AssetId], enabled: Bool) async throws
}

extension AssetActions {
    func onPinAsset(_ asset: Asset, value: Bool) {
        Task {
            do {
                let toast = try await setAssetPinned(asset, pinned: value)
                isPresentingToastMessage = ToastMessage(toast: toast)
            } catch {
                debugLog("\(Self.self) pin asset error: \(error)")
            }
        }
    }

    func onHideAsset(_ assetId: AssetId) {
        Task {
            do {
                try await setAssetsEnabled([assetId], enabled: false)
            } catch {
                debugLog("\(Self.self) hide asset error: \(error)")
            }
        }
    }

    func onAddToWallet(_ assetId: AssetId) {
        Task {
            do {
                try await setAssetsEnabled([assetId], enabled: true)
                isPresentingToastMessage = .addedToWallet()
            } catch let error as GemServiceError {
                isPresentingToastMessage = .error(error.localizedDescription)
            } catch {
                debugLog("\(Self.self) enable asset error: \(error)")
            }
        }
    }
}

@MainActor
protocol PerpetualPinActions: AnyObject {
    var isPresentingToastMessage: ToastMessage? { get set }
    func setPerpetualPinned(_ perpetual: Perpetual, pinned: Bool) async throws -> GemToast
}

extension PerpetualPinActions {
    func onSelectPinPerpetual(_ perpetualData: PerpetualData) {
        let pinned = !perpetualData.metadata.isPinned
        Task {
            do {
                let toast = try await setPerpetualPinned(perpetualData.perpetual, pinned: pinned)
                isPresentingToastMessage = ToastMessage(toast: toast)
            } catch {
                debugLog("\(Self.self) pin perpetual error: \(error)")
            }
        }
    }
}

@MainActor
protocol SearchResultActions: AssetActions, PerpetualPinActions {
    var service: any GemAssetSelectionServiceProtocol { get }
}

extension SearchResultActions {
    func setAssetPinned(_ asset: Asset, pinned: Bool) async throws -> GemToast {
        try await service.setAssetPinned(asset: asset.toGem(), pinned: pinned)
    }

    func setAssetsEnabled(_ assetIds: [AssetId], enabled: Bool) async throws {
        try await service.setAssetsEnabled(assetIds: assetIds, enabled: enabled)
    }

    func setPerpetualPinned(_ perpetual: Perpetual, pinned: Bool) async throws -> GemToast {
        try await service.setPerpetualPinned(perpetualId: perpetual.id, name: perpetual.name, pinned: pinned)
    }

    func addRecent(_ asset: Asset) {
        Task { [service] in
            do {
                try await service.addRecent(action: .open, asset: asset.toGem())
            } catch {
                debugLog("\(Self.self) add recent error: \(error)")
            }
        }
    }
}
