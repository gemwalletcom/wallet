// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct SwapDetailsView: View {
    @Environment(\.dismiss) private var dismiss
    @Bindable private var model: SwapDetailsViewModel

    @State private var isPresentingProviderSelection = false
    @State private var infoSheet: InfoSheetType?

    public init(model: Bindable<SwapDetailsViewModel>) {
        _model = model
    }

    public var body: some View {
        VStack {
            switch model.state {
            case .data: listView
            case let .error(error): List { ListItemErrorView(errorTitle: Localized.Errors.errorOccurred, error: error.asAnyError(asset: model.fromAsset)) }
            case .loading: LoadingView()
            case .noData: List { ListItemErrorView(errorTitle: nil, error: AnyError(Localized.Errors.errorOccurred)) }
            }
        }
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                Button("", systemImage: SystemImage.checkmark, action: { dismiss() })
            }
        }
        .navigationTitle(Localized.Common.details)
        .navigationBarTitleDisplayMode(.inline)
        .listSectionSpacing(.compact)
        .contentMargins([.top], .extraSmall, for: .scrollContent)
        .sheet(item: $infoSheet) {
            InfoSheetScene(type: $0)
        }
        .sheet(isPresented: $isPresentingProviderSelection) {
            SelectableListNavigationStack(
                model: model.swapProvidersViewModel,
                onFinishSelection: {
                    model.onFinishSwapProviderSelection(item: $0)
                    isPresentingProviderSelection = false
                },
                listContent: { SimpleListItemView(model: $0) },
            )
        }
    }

    private var listView: some View {
        List {
            Section {
                let view = SimpleListItemView(model: model.selectedProviderItem)
                if model.allowSelectProvider {
                    NavigationCustomLink(
                        with: view,
                    ) {
                        isPresentingProviderSelection = true
                    }
                } else {
                    view
                }
            } header: {
                Text(Localized.Common.provider)
                    .listRowInsets(.horizontalMediumInsets)
            }

            Section {
                if let rateText = model.rateText {
                    ListItemRotateView(
                        title: model.rateTitle,
                        subtitle: rateText,
                        action: model.switchRateDirection,
                    )
                }

                if let swapEstimationField = model.swapEstimationField {
                    ListItemView(field: swapEstimationField)
                }

                PriceImpactView(
                    model: model.priceImpactModel,
                    infoAction: { infoSheet = .priceImpact },
                )

                ListItemView(field: model.minReceiveField)

                ListItemView(
                    field: model.slippageField,
                    infoAction: { infoSheet = .slippage },
                )
            }
        }
    }
}
