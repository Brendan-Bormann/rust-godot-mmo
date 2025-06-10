class_name PlayerSynchronizer
extends Node

@onready var player_container = $Players
var last_players: PackedStringArray = []

func _process(_delta: float) -> void:
	sync_players();

func sync_players():
	if GlobalNetwork.active:
		var current_ids: PackedStringArray = GlobalNetwork.get_player_ids()
		
		for id: String in current_ids:
			var player = player_container.get_node_or_null(id)
			
			if player == null:
				var new_player = preload("res://player.tscn").instantiate()
				new_player.name = id
				player_container.add_child(new_player)
				GlobalNetwork.sync_player(new_player)
			else:
				GlobalNetwork.sync_player(player)
		
		for player in player_container.get_children():
			if not current_ids.has(player.name):
				player.queue_free()
	else:
		for player in player_container.get_children():
			player.queue_free()
