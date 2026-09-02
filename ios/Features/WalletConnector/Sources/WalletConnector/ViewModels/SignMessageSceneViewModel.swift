// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemSignMessagePreview
import protocol Gemstone.GemSignMessageServiceProtocol
import Components
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import WalletConnectorService

@Observable
@MainActor
public final class SignMessageSceneViewModel {
    private let service: any GemSignMessageServiceProtocol
    private let payload: SignMessagePayload
    private let confirmTransferDelegate: TransferDataCallback.ConfirmTransferDelegate
    private let preview: GemSignMessagePreview

    public var isPresentingUrl: URL?
    public var isPresentingPayloadDetails: Bool = false
    private var payloadAddressNames: [ChainAddress: AddressName] = [:]

    public init(
        service: any GemSignMessageServiceProtocol,
        payload: SignMessagePayload,
        confirmTransferDelegate: @escaping TransferDataCallback.ConfirmTransferDelegate,
    ) {
        self.service = service
        self.payload = payload
        self.confirmTransferDelegate = confirmTransferDelegate
        preview = service.preview(message: payload.message, simulation: payload.simulation.json())
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

    var messageText: String {
        preview.text
    }

    var textMessageViewModel: TextMessageViewModel {
        TextMessageViewModel(message: preview.text)
    }

    public var simulationWarnings: [SimulationWarning] {
        payload.simulation.warnings
    }

    public var payloadModel: SimulationPayloadModel {
        SimulationPayloadModel(
            chain: payload.chain,
            primaryFields: preview.primaryFields.map { $0.map() },
            secondaryFields: preview.secondaryFields.map { $0.map() },
            addressNames: payloadAddressNames,
        )
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
        let signature = try await service.sign(walletId: payload.wallet.id.id, message: payload.message)
        confirmTransferDelegate(.success(signature))
    }
}

// MARK: - Actions

public extension SignMessageSceneViewModel {
    func load() {
        Task {
            await loadPayloadAddressNamesIfNeeded()
        }
    }

    func contextMenuItems(for field: SimulationPayloadField) -> [ContextMenuItemType] {
        payloadModel.contextMenuItems(
            for: field,
            explorerLink: { BlockExplorerLink(service.addressUrl(chain: payload.chain.rawValue, address: $0)) },
            onOpenURL: { [weak self] in self?.isPresentingUrl = $0 },
        )
    }

    func onViewPayloadDetails() {
        isPresentingPayloadDetails = true
    }
}

private extension SignMessageSceneViewModel {
    func loadPayloadAddressNamesIfNeeded() async {
        guard payloadAddressNames.isEmpty, payloadModel.hasFields else { return }

        payloadAddressNames = await service.addressNames(chain: payload.chain, preview: preview)
    }
}
