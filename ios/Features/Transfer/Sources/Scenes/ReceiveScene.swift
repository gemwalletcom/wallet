// Copyright (c). Gem Wallet. All rights reserved.

import Components
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct ReceiveScene: View {
    @State private var model: ReceiveSceneViewModel

    public init(model: ReceiveSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        content
            .onChangeBindQuery(model.assetQuery, action: model.onChangeAssetData)
            .task { await model.updateAsset() }
    }

    private var content: some View {
        VStack(spacing: .large) {
            VStack {
                Spacer()
                VStack(spacing: .medium) {
                    AssetPreviewView(model: model.assetState.asset)

                    ReceiveQRCodeCard(
                        image: model.renderedImage,
                        text: model.address,
                        action: model.onCopyAddress,
                    )
                }
                Text(.init(model.warningMessage))
                    .textStyle(.subHeadline)
                    .multilineTextAlignment(.center)
                    .padding(.top, .small)
                    .frame(maxWidth: ReceiveQRCodeCard.size + .extraLarge)
                Spacer()
            }
            .frame(maxWidth: .scene.button.maxWidth)

            if model.showNetworkSelector {
                Button(action: model.onSelectNetwork) {
                    HStack {
                        ChainView(model: model.chainModel(for: model.asset.id))
                        Spacer()
                        Images.System.chevronRight
                    }
                }
                .buttonStyle(.listStyleColor(paddingVertical: .small, cornerRadius: .large, glassEffect: .disabled))
                .frame(maxWidth: .scene.button.maxWidth)
            }

            StateButton(
                text: model.copyTitle,
                image: Images.System.copy,
                action: model.onCopyAddress,
            )
            .frame(maxWidth: .scene.button.maxWidth)
        }
        .padding(.bottom, .scene.bottom)
        .frame(maxWidth: .infinity)
        .background(Colors.grayBackground)
        .navigationBarTitle(model.title)
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Button(action: model.onShareSheet) {
                    Images.System.share
                }
            }
        }
        .sheet(item: $model.isPresentingSheet) { type in
            switch type {
            case .share:
                ShareSheet(activityItems: model.activityItems(qrImage: model.renderedImage))
            case .networkSelector:
                SelectableListNavigationStack(
                    model: model.networkSelectorModel,
                    onFinishSelection: model.onFinishNetworkSelection,
                    listContent: { ChainView(model: model.chainModel(for: $0)) },
                )
                .presentationDetents([.large])
            case .copy:
                EmptyView()
            }
        }
        .copyToast(
            copy: model.copy,
            isPresenting: $model.isPresentingCopyToast,
        )
        .alertSheet($model.isPresentingAlertMessage)
        .task(id: model.address) {
            await model.onLoadImage()
        }
        .task(id: model.asset.id) {
            await model.onChangeAsset()
        }
    }
}
