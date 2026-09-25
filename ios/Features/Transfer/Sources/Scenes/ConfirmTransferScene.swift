// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
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
        ListSectionView(
            provider: model,
            content: content(for:),
        )
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
        case let .header(headerType, isReserved):
            TransactionHeaderListItemView(headerType: headerType)
                .isVisible(!isReserved)
        case let .row(row):
            GemListRowView(row: row)
        case let .recipient(model):
            AddressListItemView(model: model)
        case let .paymentAsset(model, selectable):
            NavigationCustomLink(
                with: ListItemView(model: model),
                isEnabled: selectable,
                action: self.model.onSelectPaymentAsset,
            )
        case let .swapDetails(model):
            NavigationCustomLink(
                with: SwapDetailsListView(model: model),
                action: { self.model.onSelectSwapDetails() },
            )
        case let .perpetualDetails(model):
            NavigationCustomLink(
                with: ListItemView(model: model.listItemModel),
                action: { self.model.onSelectPerpetualDetails(model) },
            )
        case let .perpetualModifyPosition(row):
            if let row {
                GemListRowView(row: row, onInfo: model.onInfo)
            }
        case let .networkFee(model, selectable):
            if selectable {
                NavigationCustomLink(
                    with: ListItemView(model: model),
                    action: self.model.onSelectFeePicker,
                )
            } else {
                ListItemView(model: model)
            }
        case let .verification(model):
            NavigationCustomLink(
                with: ListItemView(model: model),
                action: self.model.onSelectVerification,
            )
        case let .warnings(rows):
            ForEach(rows, id: \.self) { GemListRowView(row: $0) }
        case let .balanceChange(model):
            ListItemView(model: model.listItem)
        case let .payload(models):
            Group {
                SimulationPayloadFieldsContent(models: models)

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
}
