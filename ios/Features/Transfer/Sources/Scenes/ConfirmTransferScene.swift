// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemConfirmRowLink
import Primitives
import PrimitivesComponents
import Store
import Style
import Swap
import SwiftUI

public struct ConfirmTransferScene: View {
    @Environment(\.connectionStatus) private var connectionStatus

    @Bindable var model: ConfirmTransferSceneViewModel

    public init(model: ConfirmTransferSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        ListSectionView(sections: model.sections) { item in
            content(for: model.itemModel(for: item))
        }
        .contentMargins([.top], .small, for: .scrollContent)
        .listSectionSpacing(.compact)
        .safeAreaButton {
            StateButton(model.confirmButtonModel)
        }
        .frame(maxWidth: .infinity)
        .task(id: model.loadOptions) {
            await model.load()
        }
        .refreshableTimer(every: connectionStatus.refreshInterval(for: .confirm)) { @MainActor _ in
            guard model.state.screen.refreshes() else { return }
            await model.load()
        }
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .activityIndicator(isLoading: model.isConfirming, message: model.progressMessage)
        .alertSheet($model.isPresentingAlertMessage)
    }
}

// MARK: - UI Components

extension ConfirmTransferScene {
    @ViewBuilder
    private func content(for itemModel: ConfirmTransferItemModel) -> some View {
        switch itemModel {
        case let .header(header):
            TransactionHeaderListItemView(header: header.header)
                .isVisible(!header.isReserved)
        case let .row(row):
            GemListRowView(row: row)
        case let .recipient(row):
            AddressListItemView(row: row, onSelect: { model.onSelectAddress(ChainAddress(chain: Chain(core: row.chain), address: row.address)) })
        case let .paymentAsset(model, link):
            linkView(model: model, link: link, action: self.model.onSelectPaymentAsset)
        case let .message(text):
            Text(text)
        case let .swapDetails(details):
            NavigationCustomLink(
                with: SwapDetailsListView(details: details),
                action: { self.model.onSelectSwapDetails() },
            )
        case let .perpetualDetails(model):
            NavigationCustomLink(
                with: ListItemView(model: model.listItemModel),
                action: { self.model.onSelectPerpetualDetails(model) },
            )
        case let .perpetualModifyPosition(row):
            GemListRowView(row: row, onInfo: model.onInfo)
        case let .networkFee(model, link):
            linkView(model: model, link: link, action: self.model.onSelectFeePicker)
        case let .verification(model):
            NavigationCustomLink(
                with: ListItemView(model: model),
                action: self.model.onSelectVerification,
            )
        case let .warnings(rows):
            ForEach(rows, id: \.self) { GemListRowView(row: $0) }
        case let .payload(rows):
            Group {
                ForEach(Array(rows.enumerated()), id: \.offset) {
                    GemListRowView(row: $0.element, onSelectAddress: self.model.onSelectPayloadAddress)
                }

                if !self.model.secondaryPayloadFields.isEmpty {
                    NavigationCustomLink(
                        with: ListItemView(model: self.model.payloadDetailsListItem),
                        action: self.model.onSelectPayloadDetails,
                    )
                }
            }
        case let .error(title, error, onInfoAction):
            ListItemErrorView(
                errorTitle: title,
                error: error,
                infoAction: onInfoAction,
            )
        case .empty:
            EmptyView()
        }
    }

    @ViewBuilder
    private func linkView(model: ListItemModel, link: GemConfirmRowLink, action: @escaping @MainActor @Sendable () -> Void) -> some View {
        switch link {
        case .none:
            ListItemView(model: model)
        case .enabled, .disabled:
            NavigationCustomLink(
                with: ListItemView(model: model),
                isEnabled: link == .enabled,
                action: action,
            )
        }
    }
}
