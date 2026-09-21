import func Gemstone.addressCopy
import func Gemstone.privateKeyCopy
import func Gemstone.secretPhraseCopy
import GemstonePrimitives
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing
import UIKit

struct CopyTypeViewModelTests {
    @Test
    func pasteboardOptionsNil() {
        let options = CopyTypeViewModel.pasteboardOptions(expirationTime: nil)

        #expect(options.isEmpty)
    }

    @Test
    func pasteboardOptionsWithExpiration() {
        let options = CopyTypeViewModel.pasteboardOptions(expirationTime: 60)

        #expect(options[.localOnly] as? Bool == true)
        #expect(options[.expirationDate] as? Date != nil)
    }

    @Test
    func expirationTimeInternal() {
        #expect(CopyTypeViewModel(content: secretPhraseCopy(words: [])).expirationTimeInternal == 60)
        #expect(CopyTypeViewModel(content: privateKeyCopy(key: "")).expirationTimeInternal == 60)
        #expect(CopyTypeViewModel(content: addressCopy(chain: Chain.ethereum.toGem(), address: "")).expirationTimeInternal == nil)
    }
}
