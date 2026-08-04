// Copyright (c). Gem Wallet. All rights reserved.

import AddressNameService
import Components
import ExplorerService
import Foundation
import Formatters
import BigInt
import class Gemstone.MessageSigner
import Keystore
import Localization
import Preferences
import PaymentService
import Primitives
import SigningRequestService
import PrimitivesComponents
import Style
import WalletConnectorService

@Observable
@MainActor
public final class SignMessageSceneViewModel {
    private static let priceFormatter = ValueFormatter(style: .full)
    private static let feeFormatter = ValueFormatter(style: .auto)
    private static let amountFormatter = ValueFormatter(style: .auto)

    private let explorerService: ExplorerService = .standard
    private let keystore: any Keystore
    private let addressNameService: AddressNameService
    private let payload: SignMessagePayload
    private let confirmTransferDelegate: StringResultAction
    private let signer: MessageSigner
    private let plainMessage: String
    public let messageDisplayType: SignMessageDisplayType

    public var isPresentingUrl: URL?
    public var isPresentingPayloadDetails: Bool = false
    public let paymentExpiry: PaymentExpiry
    private var payloadAddressNames: [ChainAddress: AddressName] = [:]

    public init(
        keystore: any Keystore,
        addressNameService: AddressNameService,
        payload: SignMessagePayload,
        confirmTransferDelegate: @escaping StringResultAction,
    ) {
        self.keystore = keystore
        self.addressNameService = addressNameService
        self.payload = payload
        paymentExpiry = PaymentExpiry(payment: payload.payment)
        let signer = MessageSigner(message: payload.message)
        self.signer = signer
        let plainMessage = signer.plainPreview()
        self.plainMessage = plainMessage
        let messageDisplayType: SignMessageDisplayType = {
            guard let payloadPreview = try? signer.payloadPreview(simulationPayload: payload.simulation.payload.map { $0.map() }) else {
                return .text(plainMessage)
            }

            return .payload(
                primary: payloadPreview.primary.map { $0.map() },
                secondary: payloadPreview.secondary.map { $0.map() },
            )
        }()
        self.messageDisplayType = messageDisplayType
        self.confirmTransferDelegate = confirmTransferDelegate
    }

    public var priceText: String? {
        guard let price = payload.payment?.price, let value = BigInt(price.value) else {
            return .none
        }
        return Self.priceFormatter.string(value, decimals: price.decimals.asInt, currency: price.symbol)
    }

    public var expiresAt: Date? {
        payload.payment?.expiresAt
    }

    public var expiresTitle: String {
        Localized.Transfer.paymentExpiresIn
    }

    public var networkFeeTitle: String {
        Localized.Transfer.networkFee
    }

    public var networkFeeText: String? {
        guard let networkFee = payload.networkFee else {
            return .none
        }
        let display = NumericViewModel(
            data: networkFee,
            style: AmountDisplayStyle(formatter: .auto, currencyCode: Preferences.standard.currency),
        )
        return display.fiat?.text ?? display.amount.text
    }

    public var selectedQuoteItem: PaymentQuoteItem? {
        payload.payment.map { PaymentQuoteItem(quote: $0.quote, formatter: Self.amountFormatter) }
    }

    public var networkText: String {
        payload.chain.networkName
    }

    public var title: String {
        isPayment ? Localized.Transfer.paymentTitle : Localized.Transfer.reviewRequest
    }

    public var walletText: String {
        payload.wallet.name
    }

    public var buttonTitle: String {
        Localized.Transfer.confirm
    }

    public var appName: String {
        payload.appMetadata.shortName
    }

    public var appUrl: URL? {
        payload.appMetadata.url?.asURL
    }

    public var appAssetImage: AssetImage {
        AssetImage(imageURL: payload.appMetadata.iconURL)
    }

    public var merchantTitle: String {
        Localized.Transfer.merchant
    }

    public var merchantText: String? {
        isPayment ? appName : .none
    }

    public var walletAssetImage: AssetImage {
        WalletViewModel(wallet: payload.wallet).avatarImage
    }

    public var networkAssetImage: AssetImage {
        AssetIdViewModel(assetId: payload.chain.asset.id).networkAssetImage
    }

    public var appText: String {
        appName
    }

    public var appPreview: AppPreviewModel {
        guard let quote = selectedQuoteItem else {
            return AppPreviewModel(
                assetImage: appAssetImage,
                name: appName,
                subtitleSymbol: appUrl?.cleanHost(),
            )
        }
        return AppPreviewModel(
            assetImage: quote.assetImage,
            name: quote.amountText,
            subtitleSymbol: priceText,
        )
    }

    var textMessageViewModel: TextMessageViewModel {
        TextMessageViewModel(message: plainMessage)
    }

    public var simulationWarnings: [SimulationWarning] {
        payload.simulation.warnings
    }

    public var primaryPayloadFields: [SimulationPayloadField] {
        switch messageDisplayType {
        case let .payload(primaryFields, _):
            primaryFields
        case .text:
            []
        }
    }

    public var secondaryPayloadFields: [SimulationPayloadField] {
        switch messageDisplayType {
        case let .payload(_, secondaryFields):
            secondaryFields
        case .text:
            []
        }
    }

    public var hasPayload: Bool {
        !payloadFields.isEmpty
    }

    public var showsPayload: Bool {
        hasPayload && !isPayment
    }

    public var isPayment: Bool {
        payload.payment != nil
    }

    public var hasWarnings: Bool {
        !simulationWarnings.isEmpty
    }

    public var isButtonDisabled: Bool {
        paymentExpiry.isExpired || simulationWarnings.contains(where: { $0.severity == .critical })
    }

    public var buttonType: ButtonType {
        .primary(isButtonDisabled ? .disabled : .normal)
    }

    public func signMessage() async throws {
        let signature = try await keystore.signMessage(signer: signer, wallet: payload.wallet)
        confirmTransferDelegate(.success(signature))
    }
}

// MARK: - Actions

public extension SignMessageSceneViewModel {
    func fetch() {
        Task {
            await loadPayloadAddressNamesIfNeeded()
        }
    }

    func payloadFieldViewModel(for field: SimulationPayloadField) -> SimulationPayloadFieldViewModel {
        SimulationPayloadFieldViewModel(
            field: field,
            chain: payload.chain,
            addressName: payloadAddressNames[ChainAddress(chain: payload.chain, address: field.value)],
        )
    }

    func contextMenuItems(for field: SimulationPayloadField) -> [ContextMenuItemType] {
        var items = payloadFieldViewModel(for: field).contextMenuItems
        guard field.fieldType == .address else { return items }

        let link = explorerService.addressUrl(chain: payload.chain, address: field.value)
        items.append(.url(title: Localized.Transaction.viewOn(link.name), onOpen: { [weak self] in
            self?.isPresentingUrl = URL(string: link.link)
        }))
        return items
    }

    func onViewWebsite() {
        isPresentingUrl = appUrl
    }

    func onViewPayloadDetails() {
        isPresentingPayloadDetails = true
    }
}

private extension SignMessageSceneViewModel {
    func loadPayloadAddressNamesIfNeeded() async {
        guard payloadAddressNames.isEmpty else { return }
        guard !payloadFields.isEmpty else { return }

        do {
            payloadAddressNames = try await addressNameService.getAddressNames(requests: payloadAddressRequests)
        } catch {
            debugLog("payload address name lookup error: \(error)")
        }
    }

    var payloadFields: [SimulationPayloadField] {
        primaryPayloadFields + secondaryPayloadFields
    }

    var payloadAddressRequests: [ChainAddress] {
        payloadFields.compactMap {
            guard $0.fieldType == .address else {
                return nil
            }

            return ChainAddress(chain: payload.chain, address: $0.value)
        }
    }
}
