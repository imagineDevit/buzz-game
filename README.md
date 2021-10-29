#BUZZ-GAME 
__Rust version__
___________

**BUZZ-GAME** est un projet qui a pour but d'expérimenter la programmation réactive avec [JAVA](https://github.com/henriSedjame/BuzzGame.git) dans un premier temps et RUST dans un second.

---
La version de **RUST** utilisée pour le projet est la [1.56.0](https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html)

-----

### **LIBRAIRIES**

Librairies RUST utilisées                                              | Usage
---------------------------------------------------------------------- | ------------
[uuid](https://crates.io/crates/uuid)                                  | Pour la génération des ids des entités sous le format `uuid`
[serde](https://crates.io/crates/serde)                                | Pour la sérialisation / désérialisation des objets
[mobc](https://crates.io/crates/mobc)                                  | Pour la gestion des pools de connection de manière asynchrone
[mobc-postgres](https://crates.io/crates/mobc-postgres)                | Pour la connexion à la base postgres
[tokio](https://crates.io/crates/tokio)                                | Pour l'écriture de code asynchone
[async-trait](https://crates.io/crates/async-trait)                    | Pour l'écriture de `traits` contenant des functions `async`
[thiserror](https://crates.io/crates/thiserror)                        | Pour la gestion des erreurs personnalisées



