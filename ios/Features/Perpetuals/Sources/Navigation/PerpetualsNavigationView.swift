import protocol Gemstone.GemRecentActivityServiceProtocol
import class Gemstone.GemRecentActivityService
import protocol Gemstone.GemPerpetualServiceProtocol
import GemstoneServices
import Components
import GemstonePrimitives
import Primitives
import Store
import Style
import SwiftUI

public struct PerpetualsNavigationView: View {
    @State private var model: PerpetualsSceneViewModel

    public init(
        wallet: Wallet,
        perpetualService: any GemPerpetualServiceProtocol,
        observerService: any PerpetualObservable,
        recentAssetsService: any GemRecentActivityServiceProtocol,
        onSelectAssetType: @escaping (SelectAssetType) -> Void,
        onSelectAsset: @escaping (Asset) -> Void,
        onSelectPortfolio: @escaping () -> Void,
    ) {
        _model = State(
            initialValue: PerpetualsSceneViewModel(
                wallet: wallet,
                perpetualService: perpetualService,
                observerService: observerService,
                recentAssetsService: recentAssetsService,
                onSelectAssetType: onSelectAssetType,
                onSelectAsset: onSelectAsset,
                onSelectPortfolio: onSelectPortfolio,
            ),
        )
    }

    public var body: some View {
        PerpetualsScene(model: model)
            .bindQuery(model.positionsQuery, model.perpetualsQuery, model.walletBalanceQuery, model.recentModel.query)
    }
}
