// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemInfoSheet
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct SwapDetailsView: View {
    @Environment(\.dismiss) private var dismiss
    private let model: SwapDetailsViewModel

    @State private var isPresentingProviderSelection = false
    @State private var isRateInverse = false
    @State private var infoSheet: GemInfoSheet?

    public init(model: SwapDetailsViewModel) {
        self.model = model
    }

    public var body: some View {
        VStack {
            switch model.state {
            case .data: listView
            case let .error(error): List { ListItemErrorView(errorTitle: Localized.Errors.errorOccurred, error: error) }
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
            InfoSheetScene(sheet: $0)
        }
        .sheet(isPresented: $isPresentingProviderSelection) {
            SelectableListNavigationStack(
                model: model.swapProvidersViewModel,
                onFinishSelection: {
                    model.onFinishSwapProviderSelection(item: $0)
                    isPresentingProviderSelection = false
                },
                listContent: { ListItemView(model: $0.listItem) },
            )
        }
    }

    private var listView: some View {
        List {
            Section {
                let view = ListItemView(model: model.selectedProviderItem.listItem)
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
                Text(model.providerTitle)
                    .listRowInsets(.horizontalMediumInsets)
            }

            Section {
                if let rateText = model.rateText(isInverse: isRateInverse) {
                    ListItemRotateView(
                        title: model.rateTitle,
                        subtitle: rateText,
                        action: { isRateInverse.toggle() },
                    )
                }
                ForEach(Array(model.detailRows.enumerated()), id: \.offset) { _, row in
                    GemListRowView(row: row, onInfo: { infoSheet = $0.infoSheet })
                }
            }
        }
    }
}
