extends Control

onready var server_ip: LineEdit = (
	$CenterContainer/VBoxContainer/MultiplayerVBC/HBoxContainer/ServerIP
)

func _on_ConnectButton_pressed():
	get_tree().get_root().set_meta("server_ip", server_ip.text)
	get_tree().change_scene("res://Game.tscn")
