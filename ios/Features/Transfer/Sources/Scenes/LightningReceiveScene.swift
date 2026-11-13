// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemLightningReceive
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct LightningReceiveScene: View {
    @Environment(\.connectionStatus) private var connectionStatus

    @State private var model: LightningReceiveSceneViewModel

    public init(model: LightningReceiveSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        content
            .background(Colors.grayBackground)
            .navigationTitle(model.title)
            .navigationBarTitleDisplayMode(.inline)
            .debouncedTask(id: model.loadTrigger, interval: GemConstants.lightningInvoiceDebounce) { await model.onLoad() }
    }

    @ViewBuilder
    private var content: some View {
        switch model.state {
        case .loading:
            LoadingView()
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        case let .ready(receive):
            receiveView(receive)
                .refreshableTimer(every: connectionStatus.refreshInterval(for: .receive)) { _ in
                    await model.onRefresh()
                }
                .copyToast(copy: receive.copy, isPresenting: $model.isPresentingCopyToast)
                .toolbar {
                    ToolbarItem(placement: .navigationBarTrailing) {
                        ShareLink(item: receive.copy.value) { Images.System.share }
                    }
                }
        case .locked:
            ContentUnavailableView {} actions: {
                Button(model.unlockTitle) {
                    Task { await model.onUnlock() }
                }
            }
        case .unavailable:
            ContentUnavailableView {
                Text(model.unavailableTitle)
            } actions: {
                Button(model.tryAgainTitle) {
                    Task { await model.onLoad() }
                }
            }
        }
    }

    private func receiveView(_ receive: GemLightningReceive) -> some View {
        VStack(spacing: .large) {
            Spacer()
            VStack(spacing: .medium) {
                CurrencyInputView(text: $model.amount, config: model.amountInputConfig(receive))
                ReceiveQRCodeCard(image: model.renderedImage, text: receive.copy.display, action: model.onCopy)
            }
            Spacer()
            StateButton(text: model.copyTitle, image: Images.System.copy, action: model.onCopy)
                .frame(maxWidth: .scene.button.maxWidth)
        }
        .padding(.bottom, .scene.bottom)
        .frame(maxWidth: .infinity)
    }
}
