# Variables
COV_DIR = target/llvm-cov/html
PORT = 8080

.PHONY: cov clean

# Commande principale
cov:
	-fuser -k $(PORT)/tcp # Le '-' ignore l'erreur si le port est déjà libre
	# 1. Génère le rapport sans ouvrir de navigateur (évite les bugs d'OS)
	cargo llvm-cov --html
	
	# 2. Lance le serveur et affiche l'URL
	# On utilise miniserve car c'est le plus propre pour éviter les erreurs de droits
	@echo "\n🚀 Rapport prêt ! Ctrl+Clic ici --> http://localhost:$(PORT)\n"
	miniserve $(COV_DIR) --index index.html --port $(PORT)

clean:
	cargo clean
	rm -rf target/llvm-cov