// Copyright (c). Gem Wallet. All rights reserved.

import AddressNameService
import Components
import ExplorerService
import Foundation
import class Gemstone.MessageSigner
import GemstonePrimitives
import Keystore
import Localization
import Primitives
import PrimitivesComponents
import Style
import WalletConnectorService

@Observable
@MainActor
public final class SignMessageSceneViewModel {
    private let explorerService: ExplorerService = .standard
    private let keystore: any Keystore
    private let addressNameService: AddressNameService
    private let payload: SignMessagePayload
    private let confirmTransferDelegate: TransferDataCallback.ConfirmTransferDelegate
    private let signer: MessageSigner
    private let plainMessage: String
    public let messageDisplayType: SignMessageDisplayType

    public var isPresentingUrl: URL?
    public var isPresentingPayloadDetails: Bool = false
    private var payloadAddressNames: [ChainAddress: AddressName] = [:]

    public init(
        keystore: any Keystore,
        addressNameService: AddressNameService,
        payload: SignMessagePayload,
        confirmTransferDelegate: @escaping TransferDataCallback.ConfirmTransferDelegate,
    ) {
        self.keystore = keystore
        self.addressNameService = addressNameService
        self.payload = payload
        let signer = MessageSigner(message: payload.message)
        self.signer = signer
        let plainMessage = signer.plainPreview()
        self.plainMessage = plainMessage
        let messageDisplayType: SignMessageDisplayType = {
            do {
                let simulationPayload = try payload.simulation.payload.map { try $0.jsonString() }
                guard let preview = try signer.payloadPreview(simulationPayload: simulationPayload) else {
                    return .text(plainMessage)
                }
                return try .payload(
                    primary: preview.primary.map { try SimulationPayloadField($0) },
                    secondary: preview.secondary.map { try SimulationPayloadField($0) },
                )
            } catch {
                return .text(plainMessage)
            }
        }()
        self.messageDisplayType = messageDisplayType
        self.confirmTransferDelegate = confirmTransferDelegate
    }

    public var networkText: String {
        payload.chain.networkName
    }

    public var title: String {
        Localized.Transfer.reviewRequest
    }

    public var walletText: String {
        payload.wallet.name
    }

    public var buttonTitle: String {
        Localized.Transfer.confirm
    }

    public var connectionViewModel: WalletConnectionViewModel {
        WalletConnectionViewModel(connection: WalletConnection(session: payload.session, wallet: payload.wallet))
    }

    public var appName: String {
        payload.session.metadata.shortName
    }

    public var appUrl: URL? {
        payload.session.metadata.url.asURL
    }

    public var appAssetImage: AssetImage {
        AssetImage(imageURL: connectionViewModel.imageUrl)
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
        AppPreviewModel(
            assetImage: appAssetImage,
            name: appName,
            subtitleSymbol: connectionViewModel.hostText,
        )
    }

    var textMessageViewModel: TextMessageViewModel {
        TextMessageViewModel(message: plainMessage)
    }

    public var simulationWarnings: [SimulationWarning] {
        payload.simulation.warnings
    }

    public var payloadModel: SimulationPayloadModel {
        switch messageDisplayType {
        case let .payload(primaryFields, secondaryFields):
            SimulationPayloadModel(
                chain: payload.chain,
                primaryFields: primaryFields,
                secondaryFields: secondaryFields,
                addressNames: payloadAddressNames,
            )
        case .text:
            SimulationPayloadModel(chain: payload.chain, primaryFields: [], secondaryFields: [])
        }
    }

    public var hasWarnings: Bool {
        !simulationWarnings.isEmpty
    }

    public var isButtonDisabled: Bool {
        simulationWarnings.hasCritical
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

    func contextMenuItems(for field: SimulationPayloadField) -> [ContextMenuItemType] {
        payloadModel.contextMenuItems(
            for: field,
            explorerLink: { explorerService.addressUrl(chain: payload.chain, address: $0) },
            onOpenURL: { [weak self] in self?.isPresentingUrl = $0 },
        )
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
        guard payloadAddressNames.isEmpty, payloadModel.hasFields else { return }

        payloadAddressNames = await (try? addressNameService.getAddressNames(requests: payloadModel.addressRequests)) ?? [:]
    }
}
