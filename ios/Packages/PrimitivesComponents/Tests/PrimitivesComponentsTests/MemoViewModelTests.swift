// Copyright (c). Gem Wallet. All rights reserved.

@testable import PrimitivesComponents
import Testing

struct MemoViewModelTests {
    @Test
    func anEmptyMemoReadsAsThePlaceholder() {
        #expect(MemoViewModel(memo: nil).formattedMemo == MemoViewModel(memo: "").formattedMemo)
        #expect(MemoViewModel(memo: nil).formattedMemo != "")
    }

    @Test
    func aMemoIsShownAsWritten() {
        #expect(MemoViewModel(memo: "order 42").formattedMemo == "order 42")
    }
}
