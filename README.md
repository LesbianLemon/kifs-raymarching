# Fractal Ray Marching

## Opis
Program za izris 3D fraktalnih struktur kot so 3D Juliajeve množice, generirane v kvaternionskem prostoru (in KIFS - work in progress).

S pomočjo tehnike risanja pod imenom "raymarching" na grafični kartici lahko strukture, katerim običajno časovna zahtevnost z natančnostjo oz. številom iteracij raste eksponentno, izrišemo veliko hitreje, včasih celo s konstantno časovno zahtevnostjo.
Za preproste objekte zaradi narave grafičnih kartic, ki niso ustvarjene s to metodo v mislih, se takšen izris ne splača.
Korist pa vidimo, ko želimo izrisati nekaj bolj zapletenega, kar se preprosteje zapiše s t. i. predznačenim razdaljnim poljem (angl. signed distance field oz. SDF).
V programu to uporabimo tako, da poračunamo funkcijo razdalje do fraktala, ki ima zgolj linearno časovno zahtevnost, kar nam omogoči, da dosežemo nivoje natančnosti, ki jih z navadnimi metodami ne bi mogli.

## Navodila za uporabo
Program namestimo tako, da najprej repozitorij naložimo na računalnik z uporabo ukaza:

```console
git clone https://github.com/LesbianLemon/kifs-raymarching.git
```
Nato se pomaknemo v novonastalo mapo:

```console
cd kifs-raymarching
```

In poženemo program z uporabo `cargo`:

```console
cargo run --release
```

Odpre se nam okno aplikacije, katerega lahko upravljamo na sledeče načine:

| Dejanje            | Ukaz                     |
| ------------------ | ------------------------ |
| Povečati/zmanjšati | Miškino kolo gor/dol     |
| Premikanje         | Levi klik + premik miši  |
| Spreminjanje scene | Menu "Settings"          |