import ActivityService
import Components
import PerpetualService
import Primitives
import Store
import Style
import SwiftUI

public struct PerpetualsNavigationView: View {
    @State private var model: PerpetualsSceneViewModel

    public init(
        wallet: Wallet,
        perpetualService: PerpetualServiceable,
        observerService: any PerpetualObservable<HyperliquidSubscription>,
        activityService: ActivityService,
        onSelectAssetType: @escaping (SelectAssetType) -> Void,
        onSelectAsset: @escaping (Asset) -> Void,
    ) {
        _model = State(
            initialValue: PerpetualsSceneViewModel(
                wallet: wallet,
                perpetualService: perpetualService,
                observerService: observerService,
                activityService: activityService,
                onSelectAssetType: onSelectAssetType,
                onSelectAsset: onSelectAsset,
            ),
        )
    }

    public var body: some View {
        PerpetualsScene(model: model)
            .bindQuery(model.positionsQuery, model.perpetualsQuery, model.walletBalanceQuery, model.recentsQuery)
    }
}
