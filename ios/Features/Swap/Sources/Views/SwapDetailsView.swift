// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemInfoSheet
import struct Gemstone.GemProviderLoadingRow
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
    private let details: GemSwapDetails?
    private let providers: StateViewType<[GemProviderRow]>
    private let pendingProviders: [GemProviderLoadingRow]
    private let allowSelectProvider: Bool
    private let onSelectProvider: ((SwapProvider) -> Void)?

    @State private var isPresentingProviderSelection = false
    @State private var isRateInverse = false
    @State private var infoSheet: GemInfoSheet?

    public init(
        details: GemSwapDetails?,
        providers: StateViewType<[GemProviderRow]> = .data([]),
        pendingProviders: [GemProviderLoadingRow] = [],
        allowSelectProvider: Bool = false,
        onSelectProvider: ((SwapProvider) -> Void)? = nil,
    ) {
        self.details = details
        self.providers = providers
        self.pendingProviders = pendingProviders
        self.allowSelectProvider = allowSelectProvider
        self.onSelectProvider = onSelectProvider
    }

    public var body: some View {
        VStack {
            switch providers {
            case .data:
                if let details {
                    listView(details)
                } else {
                    SelectableListView(
                        model: .constant(providersModel),
                        onFinishSelection: onFinishProviderSelection,
                        listContent: { ListItemView(model: $0.listItem) },
                        trailingRows: pendingRows,
                    )
                }
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
        .navigationTitle(details == nil ? Localized.Buy.Providers.title : Localized.Common.details)
        .navigationBarTitleDisplayMode(.inline)
        .listSectionSpacing(.compact)
        .contentMargins([.top], .extraSmall, for: .scrollContent)
        .sheet(item: $infoSheet) {
            InfoSheetScene(sheet: $0)
        }
        .sheet(isPresented: $isPresentingProviderSelection) {
            SelectableListNavigationStack(
                model: .constant(providersModel),
                onFinishSelection: onFinishProviderSelection,
                listContent: { ListItemView(model: $0.listItem) },
                trailingRows: pendingRows,
            )
        }
    }

    private var providersModel: ProvidersViewModel {
        ProvidersViewModel(state: providers.map { .plain($0) })
    }

    private func pendingRows() -> some View {
        ForEach(pendingProviders) {
            ListItemView(model: $0.listItem)
        }
    }

    private func onFinishProviderSelection(_ rows: [GemProviderRow]) {
        if case let .swap(provider) = rows.first?.kind {
            onSelectProvider?(provider)
        }
        isPresentingProviderSelection = false
    }

    private func listView(_ details: GemSwapDetails) -> some View {
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
