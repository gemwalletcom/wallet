import protocol Gemstone.GemPerpetualServiceProtocol
import GemstoneServices
import Components
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
        recentActivityStore: RecentActivityStore,
        onSelectAssetType: @escaping (SelectAssetType) -> Void,
        onSelectAsset: @escaping (Asset) -> Void,
        onSelectPortfolio: @escaping () -> Void,
    ) {
        _model = State(
            initialValue: PerpetualsSceneViewModel(
                wallet: wallet,
                perpetualService: perpetualService,
                observerService: observerService,
                recentActivityStore: recentActivityStore,
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
