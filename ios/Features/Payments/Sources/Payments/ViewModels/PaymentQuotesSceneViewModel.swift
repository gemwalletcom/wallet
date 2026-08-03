// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import Style
import Localization
import PaymentService
import Primitives
import SigningRequestService
import PrimitivesComponents

@Observable
@MainActor
public final class PaymentQuotesSceneViewModel {
    private static let priceFormatter = ValueFormatter(style: .full)
    private static let amountFormatter = ValueFormatter(style: .short)

    private let request: PaymentQuotesRequest
    private let confirmTransferDelegate: StringResultAction

    public init(
        request: PaymentQuotesRequest,
        confirmTransferDelegate: @escaping StringResultAction,
    ) {
        self.request = request
        self.confirmTransferDelegate = confirmTransferDelegate
    }

    public var title: String {
        Localized.Transfer.paymentTitle
    }

    public var quotesTitle: String {
        Localized.Transfer.payWith
    }

    public var expiresTitle: String {
        Localized.Transfer.paymentExpiresIn
    }

    public var expiresAt: Date? {
        request.quotes.expiresAt
    }

    public var walletText: String {
        request.wallet.name
    }

    public var walletAssetImage: AssetImage {
        WalletViewModel(wallet: request.wallet).avatarImage
    }

    public var selected: PaymentQuote?
    public var isExpired: Bool = false
    public var isPresentingQuotes: Bool = false

    var quotesModel: PaymentQuotesViewModel {
        PaymentQuotesViewModel(
            state: .data(.plain(items)),
            selectedItems: items.filter { $0.id == selected?.id },
            selectionType: .checkmark,
        )
    }

    public var merchantTitle: String {
        Localized.Transfer.merchant
    }

    public var merchantText: String {
        request.quotes.merchant.name
    }

    public var merchantAssetImage: AssetImage {
        AssetImage(imageURL: request.quotes.merchant.iconUrl?.asURL)
    }

    public var buttonTitle: String {
        Localized.Common.continue
    }

    public var selectedItem: PaymentQuoteItem? {
        selected.map { item(for: $0) }
    }

    public var buttonType: ButtonType {
        .primary(isButtonDisabled ? .disabled : .normal)
    }

    public var isButtonDisabled: Bool {
        isExpired || selected == nil
    }

    public var preview: AppPreviewModel {
        AppPreviewModel(
            assetImage: AssetImage(imageURL: request.quotes.merchant.iconUrl?.asURL),
            name: priceText ?? selectedItem?.amountText ?? request.quotes.merchant.name,
            subtitleSymbol: .none,
        )
    }

    public func onSelectQuotes() {
        isPresentingQuotes = true
    }

    func onFinishQuotesSelection(items: [PaymentQuoteItem]) {
        selected = items.first?.quote
        isPresentingQuotes = false
    }

    public func awaitExpiry() async {
        guard let expiresAt else {
            return
        }
        await expiresAt.sleepUntil()
        isExpired = true
    }

    public func onConfirm() {
        guard let selected else {
            return
        }
        confirmTransferDelegate(.success(selected.id))
    }

    public func onAppear() {
        guard selected == nil else {
            return
        }
        selected = request.quotes.quotes.first
    }
}

// MARK: - Private

extension PaymentQuotesSceneViewModel {
    private var items: [PaymentQuoteItem] {
        request.quotes.quotes.map { item(for: $0) }
    }

    private func item(for quote: PaymentQuote) -> PaymentQuoteItem {
        PaymentQuoteItem(
            quote: quote,
            assetData: request.assetData(for: quote),
            formatter: Self.amountFormatter,
        )
    }

    private var priceText: String? {
        guard let price = request.quotes.price, let value = BigInt(price.value) else {
            return .none
        }
        return Self.priceFormatter.string(value, decimals: price.decimals.asInt, currency: price.symbol)
    }
}
