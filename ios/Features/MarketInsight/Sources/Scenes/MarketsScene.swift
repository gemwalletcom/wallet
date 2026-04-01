import Components
import Foundation
import Localization
import PrimitivesComponents
import SwiftUI

public struct MarketsScene: View {
    @State private var model: MarketsSceneViewModel

    public init(
        model: MarketsSceneViewModel,
    ) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            switch model.state {
            case .noData:
                Text("")
            case .loading:
                LoadingView()
            case let .data(data):
                PriceListItemView(model: data.marketCapViewModel)
            case let .error(error):
                ListItemErrorView(errorTitle: .none, error: error)
            }
        }
        .refreshable {
            await model.fetch()
        }
        .onAppear {
            Task {
                await model.fetch()
            }
        }
        .overlay {
            if model.state.isNoData {
                EmptyContentView(model: model.emptyContentModel)
            }
        }
        .navigationTitle(model.title)
    }
}
