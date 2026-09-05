// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Components
import FiatConnect
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Recents
import Style
import SwiftUI
import Transfer
import struct Gemstone.GemTransferData

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
            .onChange(of: model.assetSelection, onChangeAssetSelection)
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
                                type: .asset(asset: input.asset.map()),
                                recipient: recipient,
                                onRecipientDataAction: {
                                    navigationPath.append($0)
                                },
                                onTransferAction: {
                                    navigationPath.append($0)
                                },
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
                                onTransferAction: {
                                    navigationPath.append($0)
                                },
                            ),
                        )
                    case .withdraw:
                        AmountNavigationView(
                            model: viewModelFactory.amountScene(
                                input: AmountInput(type: .withdraw, asset: input.asset),
                                wallet: model.wallet,
                                onTransferAction: {
                                    navigationPath.append($0)
                                },
                            ),
                        )
                    case .manage, .priceAlert, .swap:
                        EmptyView()
                    }
                }
            }
            .navigationBarTitleDisplayMode(.inline)
            .navigationDestination(for: GemTransferData.self) { data in
                ConfirmTransferNavigationView(
                    model: viewModelFactory.confirmTransferScene(
                        wallet: model.wallet,
                        data: data,
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
            .presentationDetentsForCurrentDeviceSize(expandable: true)
            .presentationDragIndicator(.visible)
            .presentationBackground(Colors.grayBackground)
        }
        .recentAssetsSheet(model: model.recentModel, onSelect: model.onSelectRecent)
    }
}

// MARK: - Actions

extension SelectAssetSceneNavigationStack {
    private func onSelectFilter() {
        isPresentingFilteringView.toggle()
    }

    private func onChangeAssetSelection(_: SelectAssetInput?, new: SelectAssetInput?) {
        guard let new else { return }
        model.assetSelection = nil
        navigationPath.append(new)
    }
}
