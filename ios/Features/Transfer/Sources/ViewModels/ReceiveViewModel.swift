import Components
import Foundation
import func Gemstone.addressCopy
import struct Gemstone.GemReceiveNetworks
import protocol Gemstone.GemReceiveServiceProtocol
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import SwiftUI

@Observable
@MainActor
public final class ReceiveViewModel: Sendable {
    var qrSize: CGFloat {
        UIDevice.current.userInterfaceIdiom == .pad ? 180 : 260
    }

    private(set) var assetModel: AssetViewModel
    private(set) var address: String

    var presentation: ReceivePresentationType?
    var renderedImage: UIImage?
    var isPresentingAlertMessage: AlertMessage?

    private let wallet: Wallet
    private let service: any GemReceiveServiceProtocol
    private let generator = QRCodeGenerator()
    let networks: GemReceiveNetworks
    private(set) var selectNetworkTask: Task<Void, Never>?

    private init(
        asset: Asset,
        associations: [AssetAssociation],
        wallet: Wallet,
        address: String,
        service: any GemReceiveServiceProtocol,
    ) {
        assetModel = AssetViewModel(asset: asset)
        self.wallet = wallet
        self.address = address
        self.service = service
        networks = service.networks(
            assetId: asset.id.identifier,
            associations: associations.map(\.assetId.identifier),
            wallet: wallet.toGem(),
        )
    }

    public convenience init(assetData: AssetData, wallet: Wallet, service: any GemReceiveServiceProtocol) {
        self.init(
            asset: assetData.asset,
            associations: assetData.associations,
            wallet: wallet,
            address: assetData.account.address,
            service: service,
        )
    }

    public convenience init(assetAddress: AssetAddress, wallet: Wallet, service: any GemReceiveServiceProtocol) {
        self.init(
            asset: assetAddress.asset,
            associations: [],
            wallet: wallet,
            address: assetAddress.address,
            service: service,
        )
    }

    var title: String {
        Localized.Wallet.receive
    }

    var copyTitle: String {
        Localized.Common.copy
    }

    var warningMessage: String {
        service.warnings(chain: assetModel.asset.chain.rawValue)
            .map { $0.text(asset: assetModel) }
            .joined(separator: " ")
    }

    var copyModel: CopyTypeViewModel {
        CopyTypeViewModel(content: addressCopy(chain: assetModel.asset.chain.toGem(), address: address))
    }

    var showNetworkSelector: Bool {
        networks.showsSelector
    }

    var networkSelectorModel: ReceiveNetworkSelectorViewModel {
        ReceiveNetworkSelectorViewModel(
            assetIds: networks.assetIds.map { AssetId(core: $0) },
        )
    }

    var isPresentingSheet: ReceivePresentationType? {
        get {
            switch presentation {
            case .share, .networkSelector: presentation
            case .copy, nil: nil
            }
        }
        set {
            presentation = newValue
        }
    }

    var isPresentingCopyToast: Bool {
        get {
            if case .copy = presentation {
                return true
            }
            return false
        }
        set {
            presentation = newValue ? .copy : nil
        }
    }

    func activityItems(qrImage: UIImage?) -> [Any] {
        if let qrImage {
            return [qrImage, address]
        }
        return [address]
    }

    private func selectNetwork(assetId: AssetId) async {
        do {
            let asset = try await service.asset(assetId: assetId.identifier).toPrimitives()
            let account = try wallet.account(for: asset.chain)
            try Task.checkCancellation()
            assetModel = AssetViewModel(asset: asset)
            address = account.address
        } catch {
            guard !error.isCancelled else { return }
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }

    private func generateQRCode() async -> UIImage? {
        await generator.generate(
            from: address,
            size: CGSize(
                width: qrSize,
                height: qrSize,
            ),
            logo: UIImage.name("logo-dark"),
        )
    }
}

// MARK: - Actions

extension ReceiveViewModel {
    func onChangeAsset() async {
        do {
            try await service.enableAsset(walletId: wallet.id.id, assetId: assetModel.asset.id.identifier)
        } catch {
            debugLog("ReceiveViewModel enableAsset error: \(error)")
        }
    }

    func onSelectNetwork() {
        presentation = .networkSelector
    }

    func onFinishNetworkSelection(_ items: [ReceiveNetworkItem]) {
        presentation = nil
        guard let assetId = items.first?.assetId, assetId != assetModel.asset.id else { return }

        selectNetworkTask?.cancel()
        selectNetworkTask = Task { await selectNetwork(assetId: assetId) }
    }

    func onShareSheet() {
        presentation = .share
    }

    func onCopyAddress() {
        presentation = .copy
    }

    func onLoadImage() async {
        renderedImage = nil
        renderedImage = await generateQRCode()
    }
}
