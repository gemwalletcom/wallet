// Copyright (c). Gem Wallet. All rights reserved.

import Components
import ExplorerService
import Foundation
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct TransactionNavigationView: View {
    @State private var model: TransactionSceneViewModel

    public init(model: TransactionSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        TransactionScene(
            model: model,
        )
        .bindQuery(model.query)
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Button(action: model.onSelectShare) {
                    Images.System.share
                }
            }
        }
        .sheet(item: $model.isPresentingTransactionSheet) { sheetType in
            switch sheetType {
            case .share:
                ShareSheet(activityItems: [model.explorerURL.absoluteString])
            case .feeDetails:
                NetworkFeeSheet(model: model.feeDetailsViewModel)
            case let .info(infoType):
                InfoSheetScene(type: infoType)
            }
        }
    }
}
