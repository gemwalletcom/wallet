// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import Components
import FiatConnect
import GemstonePrimitives
import InfoSheet
import Perpetuals
import Primitives
import PrimitivesComponents
import Style
import Swap
import SwiftUI
import Transfer

struct ConfirmTransferNavigationView: View {
    @Environment(\.viewModelFactory) private var viewModelFactory
    @Environment(\.assetsEnabler) private var assetsEnabler

    @State var model: ConfirmTransferSceneViewModel

    var body: some View {
        ConfirmTransferScene(model: model)
            .sheet(item: $model.isPresentingSheet) {
                switch $0 {
                case let .info(type):
                    InfoSheetScene(type: type)
                case let .url(url):
                    SFSafariView(url: url)
                case .networkFeeSelector:
                    NetworkFeeSheet(model: model.feeModel)
                case .payloadDetails:
                    NavigationStack {
                        SimulationPayloadDetailsScene(
                            primaryFields: model.primaryPayloadFields,
                            secondaryFields: model.secondaryPayloadFields,
                            fieldViewModel: model.payloadFieldViewModel(for:),
                            contextMenuItems: model.contextMenuItems(for:),
                        )
                        .presentationDetents([.large])
                        .presentationBackground(Colors.grayBackground)
                    }
                case let .fiatConnect(assetAddress, wallet, amount):
                    NavigationStack {
                        FiatConnectNavigationView(
                            model: viewModelFactory.fiatScene(assetAddress: assetAddress, wallet: wallet, amount: amount),
                        )
                        .navigationBarTitleDisplayMode(.inline)
                        .toolbarDismissItem(type: .close, placement: .topBarLeading)
                    }
                case let .getAsset(asset, buyAmount):
                    GetAssetNavigationStack(
                        asset: asset,
                        buyAmount: buyAmount,
                        model: model,
                        viewModelFactory: viewModelFactory,
                        assetsEnabler: assetsEnabler,
                    )
                case let .selectedAsset(input, wallet):
                    SelectedAssetNavigationStack(
                        input: input,
                        wallet: wallet,
                        onComplete: { model.isPresentingSheet = nil },
                    )
                case .swapDetails:
                    if case let .swapDetails(model) = model.detailsViewModel.itemModel {
                        NavigationStack {
                            SwapDetailsView(model: Bindable(model))
                                .presentationDetentsForCurrentDeviceSize(expandable: true)
                                .presentationBackground(Colors.grayBackground)
                        }
                    }
                case let .perpetualDetails(model):
                    NavigationStack {
                        PerpetualDetailsView(model: model)
                            .presentationDetentsForCurrentDeviceSize(expandable: true)
                            .presentationBackground(Colors.grayBackground)
                    }
                case let .addContact(action):
                    AddContactNavigationView(action: action)
                }
            }
    }
}

private struct GetAssetNavigationStack: View {
    private static let optionsDetent = PresentationDetent.height(360)

    let asset: Asset
    let buyAmount: Int?
    let model: ConfirmTransferSceneViewModel
    let viewModelFactory: ViewModelFactory
    let assetsEnabler: any AssetsEnabler

    @State private var selectedAction: GetAssetAction?
    @State private var actionNavigationPath = NavigationPath()

    var body: some View {
        NavigationStack {
            GetAssetScene(
                asset: asset,
                onSelect: {
                    actionNavigationPath = NavigationPath()
                    selectedAction = $0
                },
            )
            .toolbarDismissItem(type: .close, placement: .topBarLeading)
            .presentationDetents([Self.optionsDetent])
            .presentationBackground(Colors.grayBackground)
            .sheet(item: $selectedAction) { action in
                NavigationStack(path: $actionNavigationPath) {
                    destination(for: action)
                        .toolbarDismissItem(type: .close, placement: .topBarLeading)
                        .navigationBarTitleDisplayMode(.inline)
                        .navigationDestination(for: TransferData.self) { data in
                            ConfirmTransferNavigationView(
                                model: viewModelFactory.confirmTransferScene(
                                    wallet: model.assetAcquisitionWallet,
                                    data: data,
                                    onComplete: { model.isPresentingSheet = nil },
                                ),
                            )
                        }
                }
                .presentationDetents([.large])
                .presentationBackground(Colors.grayBackground)
            }
        }
    }

    @ViewBuilder
    private func destination(for type: GetAssetAction) -> some View {
        switch type {
        case .buy:
            FiatConnectNavigationView(
                model: viewModelFactory.fiatScene(
                    assetAddress: model.assetAddress(asset),
                    wallet: model.assetAcquisitionWallet,
                    amount: buyAmount,
                ),
            )
        case .swap:
            SwapNavigationView(
                model: viewModelFactory.swapScene(
                    input: SwapInput(
                        wallet: model.assetAcquisitionWallet,
                        pairSelector: SwapPairSelectorViewModel(
                            fromAssetId: model.swapFromAsset(to: asset).id,
                            toAssetId: asset.id,
                        ),
                    ),
                    onSwap: { actionNavigationPath.append($0) },
                ),
            )
        case .receive:
            ReceiveScene(
                model: ReceiveViewModel(
                    assetModel: AssetViewModel(asset: asset),
                    wallet: model.assetAcquisitionWallet,
                    address: model.assetAddress(asset).address,
                    assetsEnabler: assetsEnabler,
                ),
            )
        }
    }
}
