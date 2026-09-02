import GemstoneServices
import Components
import GemstonePrimitives
import Primitives
import Store
import Style
import SwiftUI

public struct PerpetualsNavigationView: View {
    @State private var model: PerpetualsSceneViewModel

    public init(model: PerpetualsSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        PerpetualsScene(model: model)
            .bindQuery(model.positionsQuery, model.perpetualsQuery, model.walletBalanceQuery, model.recentModel.query)
    }
}
