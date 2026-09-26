// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemInfoSheet
import struct Gemstone.GemProviderRow
import struct Gemstone.GemSwapDetails
import enum Gemstone.SwapProvider
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct SwapDetailsView: View {
    @Environment(\.dismiss) private var dismiss
    private let details: GemSwapDetails
    private let providers: StateViewType<[GemProviderRow]>
    private let allowSelectProvider: Bool
    private let onSelectProvider: ((SwapProvider) -> Void)?

    @State private var isPresentingProviderSelection = false
    @State private var isRateInverse = false
    @State private var infoSheet: GemInfoSheet?

    public init(
        details: GemSwapDetails,
        providers: StateViewType<[GemProviderRow]> = .data([]),
        allowSelectProvider: Bool = false,
        onSelectProvider: ((SwapProvider) -> Void)? = nil,
    ) {
        self.details = details
        self.providers = providers
        self.allowSelectProvider = allowSelectProvider
        self.onSelectProvider = onSelectProvider
    }

    public var body: some View {
        VStack {
            switch providers {
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
                model: ProvidersViewModel(state: providers.map { .plain($0) }),
                onFinishSelection: {
                    if case let .swap(provider) = $0.first?.kind {
                        onSelectProvider?(provider)
                    }
                    isPresentingProviderSelection = false
                },
                listContent: { ListItemView(model: $0.listItem) },
            )
        }
    }

    private var listView: some View {
        List {
            Section {
                let view = ListItemView(model: details.provider.listItem)
                if allowSelectProvider {
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
                if let rateText = details.rateText(isInverse: isRateInverse) {
                    ListItemRotateView(
                        title: Localized.Buy.rate,
                        subtitle: rateText,
                        action: { isRateInverse.toggle() },
                    )
                }
                ForEach(Array(details.rows.enumerated()), id: \.offset) { _, row in
                    GemListRowView(row: row, onInfo: { infoSheet = $0.infoSheet })
                }
            }
        }
    }
}
