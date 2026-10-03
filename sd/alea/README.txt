Alea microSD contents / microSD の内容

Copy this "alea" folder to the root of a FAT32 microSD card.
この alea フォルダを、FAT32 でフォーマットした microSD のルートにコピーします。

/alea/
  config.json            shake thresholds etc. / シェイク閾値など
  fonts/                 Japanese fonts / 日本語フォント
  tarot/img/NN.a2b       card images and back.a2b (tools/convert_cards.py)
                         カード画像と裏面（tools/convert_cards.py で生成）
  tarot/cap/NN.a1b       card names / カード名（tools/render_tarot.py で生成）
  tarot/word/NN.a1b      keywords / キーワード（tools/render_tarot.py で生成）
                         The tarot files are generated and not in the repository.
                         タロットのファイルは生成物で、リポジトリには含めない。
  iching/hexagrams.json  64 hexagrams / 64卦
  rune/runes.json        24 runes / 24文字

See DESIGN.md section 7 for details. / 詳細は DESIGN.md §7 を参照。
