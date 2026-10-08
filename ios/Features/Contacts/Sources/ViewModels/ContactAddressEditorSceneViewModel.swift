// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import class Gemstone.GemChainService
import enum Gemstone.GemContactAddressField
import struct Gemstone.GemContactAddressInput
import struct Gemstone.GemContactAddressSession
import protocol Gemstone.GemContactEditorServiceProtocol
import protocol Gemstone.GemNameServiceProtocol
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI
import UIKit

@Observable
@MainActor
public final class ContactAddressEditorSceneViewModel {
    public enum Mode: Identifiable {
        case add
        case edit(ContactAddress)

        public var id: String {
            switch self {
            case .add: "add"
            case let .edit(address): address.id
            }
        }

        var contactAddress: ContactAddress? {
            switch self {
            case .add: nil
            case let .edit(address): address
            }
        }
    }

    private let chains: [Chain]
    private let service: any GemContactEditorServiceProtocol
    private let onComplete: (GemContactAddressInput) -> Void

    var addressInputModel: AddressInputViewModel
    private(set) var session: GemContactAddressSession
    var isPresentingScanner = false

    public init(
        service: any GemContactEditorServiceProtocol,
        nameService: any GemNameServiceProtocol,
        contactId: String,
        mode: Mode,
        onComplete: @escaping (GemContactAddressInput) -> Void,
    ) {
        chains = GemChainService.shared.chainRows(chains: nil, query: .empty).map { Chain(core: $0.chain) }
        self.service = service
        self.onComplete = onComplete
        title = Localized.Common.address

        let session = service.newAddressSession(contactId: contactId, existing: mode.contactAddress?.toGem())
        self.session = session
        addressInputModel = AddressInputViewModel(
            chain: Chain(core: session.chain),
            nameService: nameService,
            placeholder: title,
        )

        if let address = mode.contactAddress {
            addressInputModel.text = address.address
        }
    }

    let title: String

    var chain: Chain {
        addressInputModel.chain
    }

    var memo: String {
        get { session.memo }
        set { session = session.onMemoChanged(memo: newValue) }
    }

    var fields: [GemContactAddressField] {
        session.fields
    }

    var networkTitle: String {
        GemContactAddressField.network.title
    }

    var memoTitle: String {
        GemContactAddressField.memo.title
    }

    var networkSelectorModel: NetworkSelectorViewModel {
        NetworkSelectorViewModel(
            state: .data(.plain(chains)),
            selectedItems: [chain],
            selectionType: .checkmark,
            title: GemContactAddressField.network.title,
        )
    }

    var buttonState: ButtonState {
        addressInputModel.isValid ? .normal : .disabled
    }

    private var input: GemContactAddressInput {
        session.input(address: addressInputModel.resolvedAddress)
    }
}

// MARK: - Actions

extension ContactAddressEditorSceneViewModel {
    func onSelectChain(_ chain: Chain) {
        addressInputModel.chain = chain
        session = session.onChainChanged(chain: chain.rawValue)
    }

    func onSelectScan() {
        isPresentingScanner = true
    }

    func onSelectPaste() {
        guard let text = UIPasteboard.general.string else { return }
        onScan(text)
    }

    func onScan(_ result: String) {
        let scan = service.scannedAddress(input: result)
        addressInputModel.update(text: scan.address)
        session = session.onScanned(scan: scan)
    }

    func complete() {
        guard addressInputModel.validate() else { return }
        onComplete(input)
    }
}
