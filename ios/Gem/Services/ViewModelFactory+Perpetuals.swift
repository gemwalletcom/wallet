// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import GemstoneServices
import Perpetuals
import Primitives
import PrimitivesComponents
import Store
import SwiftUI
import class Gemstone.GemPerpetualDetailsService
import enum Gemstone.GemPerpetualPositionAction

extension ViewModelFactory {
    @MainActor
    public func perpetualsScene(
        wallet: Wallet,
        onSelectAssetType: @escaping (SelectAssetType) -> Void,
        onSelectAsset: @escaping (Asset) -> Void,
        onSelectPortfolio: @escaping () -> Void,
    ) -> PerpetualsSceneViewModel {
        PerpetualsSceneViewModel(
            wallet: wallet,
            service: perpetualService,
            observerService: hyperliquidObserverService,
            recentAssetsService: recentAssetsService,
            onSelectAssetType: onSelectAssetType,
            onSelectAsset: onSelectAsset,
            onSelectPortfolio: onSelectPortfolio,
        )
    }

    @MainActor
    public func perpetualScene(
        asset: Asset,
        wallet: Wallet,
        onTransferData: TransferDataAction,
        onPerpetualPosition: ((GemPerpetualPositionAction) -> Void)?,
    ) -> PerpetualSceneViewModel {
        PerpetualSceneViewModel(
            wallet: wallet,
            asset: asset,
            service: GemPerpetualDetailsService(perpetuals: perpetualService, transactions: transactionsService, preferences: preferencesService, session: walletSessionService),
            observerService: hyperliquidObserverService,
            onTransferData: onTransferData,
            onPerpetualPosition: onPerpetualPosition,
        )
    }
}
