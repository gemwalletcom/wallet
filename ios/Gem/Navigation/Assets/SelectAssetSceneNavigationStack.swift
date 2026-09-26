// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Components
import FiatConnect
import struct Gemstone.GemTransferData
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Style
import SwiftUI
import Transfer

struct SelectAssetSceneNavigationStack: View {
    @Environment(\.viewModelFactory) private var viewModelFactory
    @Environment(\.dismiss) private var dismiss

    @State private var isPresentingFilteringView: Bool = false

    @State private var model: SelectAssetViewModel
    @State private var navigationPath = NavigationPath()

    init(model: SelectAssetViewModel) {
        _model = State(wrappedValue: model)
    }

    var body: some View {
        NavigationStack(path: $navigationPath) {
            SelectAssetScene(
                model: model,
            )
            .onChange(of: model.route, onChangeRoute)
            .toolbar {
                ToolbarDismissItem(
                    type: .close,
                    placement: .topBarLeading,
                )
                if model.showFilter {
                    ToolbarItem(placement: .navigationBarTrailing) {
                        FilterButton(
                            isActive: model.filterModel.isAnyFilterSpecified,
                            action: onSelectFilter,
                        )
                    }
                }
                if model.showAddToken {
                    ToolbarItem(placement: .navigationBarTrailing) {
                        Button {
                            model.isPresentingAddToken = true
                        } label: {
                            Images.System.plus
                        }
                    }
                }
            }
            .navigationDestination(for: SelectAssetInput.self) { input in
                Group {
                    switch input.type {
                    case let .send(recipient):
                        RecipientNavigationView(
                            model: viewModelFactory.recipientScene(
                                wallet: model.wallet,
                                asset: input.asset,
                                type: .asset(asset: input.asset.toGem()),
                                recipient: recipient,
                                onNavigate: navigate,
                            ),
                        )
                    case .receive:
                        ReceiveScene(model: viewModelFactory.receiveScene(assetData: input.assetData, wallet: model.wallet))
                    case .buy:
                        FiatConnectNavigationView(
                            model: viewModelFactory.fiatScene(
                                assetAddress: input.assetAddress,
                                wallet: model.wallet,
                            ),
                        )
                    case .deposit:
                        AmountNavigationView(
                            model: viewModelFactory.amountScene(
                                input: AmountInput(type: .deposit, asset: input.asset),
                                wallet: model.wallet,
                                onTransferAction: { navigate(to: .confirm($0)) },
                            ),
                        )
                    case .withdraw:
                        AmountNavigationView(
                            model: viewModelFactory.amountScene(
                                input: AmountInput(type: .withdraw, asset: input.asset),
                                wallet: model.wallet,
                                onTransferAction: { navigate(to: .confirm($0)) },
                            ),
                        )
                    case .manage, .priceAlert, .swap, .payment:
                        EmptyView()
                    }
                }
            }
            .navigationBarTitleDisplayMode(.inline)
            .navigationDestination(for: AmountInput.self) { amount in
                AmountNavigationView(
                    model: viewModelFactory.amountScene(
                        input: amount,
                        wallet: model.wallet,
                        onTransferAction: { navigate(to: .confirm($0)) },
                    ),
                )
            }
            .navigationDestination(for: ConfirmTransferInput.self) { confirm in
                ConfirmTransferNavigationView(
                    model: viewModelFactory.confirmTransferScene(
                        wallet: model.wallet,
                        data: confirm.data,
                        onComplete: { dismiss() },
                    ),
                )
            }
        }
        .sheet(isPresented: $model.isPresentingAddToken) {
            AddAssetNavigationStack(wallet: model.wallet)
        }
        .sheet(isPresented: $isPresentingFilteringView) {
            NavigationStack {
                AssetsFilterScene(model: $model.filterModel)
            }
            .sheetPresentation(.forCurrentDeviceSize(expandable: true), dragIndicator: .visible)
        }
        .recentAssetsSheet(model: model.recentModel, onSelect: model.onSelectRecent)
    }
}

// MARK: - Actions

extension SelectAssetSceneNavigationStack {
    private func onSelectFilter() {
        isPresentingFilteringView.toggle()
    }

    private func onChangeRoute(_: SelectAssetRoute?, new: SelectAssetRoute?) {
        guard let new else { return }
        model.route = nil
        switch new {
        case let .asset(input): navigationPath.append(input)
        case let .transfer(route): navigate(to: route)
        }
    }

    private func navigate(to route: TransferRoute) {
        switch route {
        case let .amount(input): navigationPath.append(input)
        case let .confirm(data): navigationPath.append(ConfirmTransferInput(data: data))
        }
    }
}
