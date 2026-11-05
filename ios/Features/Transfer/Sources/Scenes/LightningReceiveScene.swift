// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.LightningInvoice
import PrimitivesComponents
import Style
import SwiftUI

public struct LightningReceiveScene: View {
    @Environment(\.scenePhase) private var scenePhase

    @State private var model: LightningReceiveSceneViewModel
    @FocusState private var isAmountFocused: Bool

    public init(model: LightningReceiveSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        content
            .background(Colors.grayBackground)
            .navigationTitle(model.title)
            .navigationBarTitleDisplayMode(.inline)
            .toast(message: $model.isPresentingToastMessage)
            .task { await model.open() }
    }

    @ViewBuilder
    private var content: some View {
        switch model.state {
        case .opening:
            LoadingView()
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        case .amount:
            amountView
        case let .invoice(invoice):
            invoiceView(invoice)
                .task(id: scenePhase) {
                    guard scenePhase == .active else { return }
                    await model.waitForPayment(invoice)
                }
                .id(invoice.paymentHash)
                .copyToast(copy: model.copy(invoice), isPresenting: $model.isPresentingCopyToast)
                .toolbar {
                    ToolbarItem(placement: .navigationBarTrailing) {
                        ShareLink(item: invoice.invoice) { Images.System.share }
                    }
                }
        case let .expired(invoice):
            expiredView(invoice)
        case .unlockRequired, .unavailable:
            ContentUnavailableView {
                Label(model.asset.name, systemImage: model.unavailableImage)
            } description: {
                Text(model.unavailableMessage)
            } actions: {
                Button(model.retryTitle) {
                    Task { await model.onRetry() }
                }
            }
        }
    }

    private var amountView: some View {
        List {
            CurrencyInputView(text: $model.amount, config: model.amountInputConfig)
                .padding(.top, .medium)
                .listGroupRowStyle()
                .focused($isAmountFocused)

            Section {} footer: {
                Text(model.amountHint)
                    .frame(maxWidth: .infinity)
                    .multilineTextAlignment(.center)
            }
        }
        .contentMargins([.top], .zero, for: .scrollContent)
        .safeAreaButton {
            StateButton(text: model.createInvoiceTitle, state: model.createInvoiceButtonState) {
                Task { await model.onCreateInvoice() }
            }
        }
        .onAppear { isAmountFocused = true }
    }

    private func invoiceView(_ invoice: LightningInvoice) -> some View {
        VStack(spacing: .large) {
            Spacer()
            VStack(spacing: .medium) {
                Text(model.invoiceAmountText(invoice))
                    .font(.app.title2)
                    .foregroundStyle(Colors.black)

                ReceiveQRCodeCard(
                    image: model.renderedImage,
                    text: model.invoiceText(invoice),
                    action: model.onCopyInvoice,
                )

                Text(model.invoiceHint(invoice))
                    .textStyle(.subHeadline)
                    .multilineTextAlignment(.center)
                    .fixedSize(horizontal: false, vertical: true)
                    .frame(maxWidth: ReceiveQRCodeCard.size + .extraLarge)

                if model.isUnlockRequired {
                    VStack(spacing: .small) {
                        Text(model.unlockHint)
                            .textStyle(.subHeadline)
                            .multilineTextAlignment(.center)
                        Button(model.unlockTitle) {
                            Task { await model.onUnlock() }
                        }
                        .font(.callout.weight(.semibold))
                        .foregroundStyle(Colors.blue)
                    }
                    .frame(maxWidth: ReceiveQRCodeCard.size + .extraLarge)
                }

                Button(model.changeAmountTitle, action: model.onChangeAmount)
                    .font(.callout.weight(.semibold))
                    .foregroundStyle(Colors.blue)
            }
            Spacer()
            StateButton(text: model.copyTitle, image: Images.System.copy, action: model.onCopyInvoice)
                .frame(maxWidth: .scene.button.maxWidth)
        }
        .padding(.bottom, .scene.bottom)
        .frame(maxWidth: .infinity)
    }

    private func expiredView(_ invoice: LightningInvoice) -> some View {
        VStack(spacing: .medium) {
            Text(model.invoiceAmountText(invoice))
                .font(.app.title2)
                .foregroundStyle(Colors.black)

            Text(model.expiredText)
                .textStyle(.subHeadline)

            Button(model.changeAmountTitle, action: model.onChangeAmount)
                .font(.callout.weight(.semibold))
                .foregroundStyle(Colors.blue)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
