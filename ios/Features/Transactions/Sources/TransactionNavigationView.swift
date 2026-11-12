// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import InfoSheet
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
        content
            .bindQuery(model.query)
            .task { await model.load() }
            .onChange(of: model.details?.transaction.recordId, initial: true) { _, recordId in
                model.onRecordChange(recordId)
            }
            .toolbar {
                if model.explorerLink != nil {
                    ToolbarItem(placement: .navigationBarTrailing) {
                        Button(action: model.onSelectShare) {
                            Images.System.share
                        }
                    }
                }
            }
            .sheet(item: $model.isPresentingTransactionSheet) { sheetType in
                switch sheetType {
                case .share:
                    if let explorerLink = model.explorerLink {
                        ShareSheet(activityItems: [explorerLink])
                    }
                case .feeDetails:
                    if let feeDetailsViewModel = model.feeDetailsViewModel {
                        NetworkFeeSheet(model: feeDetailsViewModel)
                    }
                case let .info(infoType):
                    InfoSheetScene(sheet: infoType)
                }
            }
    }

    @ViewBuilder
    private var content: some View {
        if model.details != nil {
            TransactionScene(model: model)
        } else {
            switch model.loadState {
            case let .error(error):
                StateErrorView(error: error) { await model.load() }
            case .loading, .data, .noData:
                CenterLoadingView()
            }
        }
    }
}
