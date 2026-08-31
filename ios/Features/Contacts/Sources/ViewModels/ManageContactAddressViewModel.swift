// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemManageContactServiceProtocol
import Components
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI
import Validators

@Observable
@MainActor
public final class ManageContactAddressViewModel {
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

    public struct Input: Sendable {
        public let chain: Chain
        public let address: String
        public let memo: String?
        public let replacingId: String?
    }

    private let mode: Mode
    private let service: any GemManageContactServiceProtocol
    private let onComplete: (Input) -> Void

    var addressInputModel: AddressInputViewModel
    var memo: String = ""
    var isPresentingScanner = false

    public init(
        service: any GemManageContactServiceProtocol,
        mode: Mode,
        onComplete: @escaping (Input) -> Void,
    ) {
        self.mode = mode
        self.service = service
        self.onComplete = onComplete
        title = Localized.Common.address

        let addressService = service.addresses()
        let chain = mode.contactAddress?.chain ?? service.defaultContactChain
        addressInputModel = AddressInputViewModel(
            chain: chain,
            nameService: service.names(),
            placeholder: title,
            addressService: addressService,
            validators: [.required(requireName: title), .address(Asset(chain), addressService: addressService)],
        )

        if let address = mode.contactAddress {
            addressInputModel.text = address.address
            memo = address.memo ?? ""
        }
    }

    let title: String
    var buttonTitle: String {
        Localized.Transfer.confirm
    }

    var networkTitle: String {
        Localized.Transfer.network
    }

    var memoTitle: String {
        Localized.Transfer.memo
    }

    var chain: Chain {
        addressInputModel.chain
    }

    var showMemo: Bool {
        chain.isMemoSupported
    }

    var networkSelectorModel: NetworkSelectorViewModel {
        NetworkSelectorViewModel(
            state: .data(.plain(Chain.allCases)),
            selectedItems: [chain],
            selectionType: .checkmark,
            chainService: service.chains(),
        )
    }

    var buttonState: ButtonState {
        addressInputModel.isValid ? .normal : .disabled
    }

    private var input: Input {
        Input(
            chain: chain,
            address: addressInputModel.resolvedAddress,
            memo: memo,
            replacingId: mode.contactAddress?.id,
        )
    }
}

// MARK: - Actions

extension ManageContactAddressViewModel {
    func onSelectChain(_ chain: Chain) {
        addressInputModel.chain = chain
        memo = ""
    }

    func onSelectScan() {
        isPresentingScanner = true
    }

    func onHandleScan(_ result: String) {
        addressInputModel.update(text: result)
    }

    func complete() {
        guard addressInputModel.validate() else { return }
        onComplete(input)
    }
}
