INSERT INTO 
	providers
(
	id,
	name,
    description,
    url,
    is_active

)
VALUES
(   gen_random_uuid(),
	'FeverUp',
	'This is the provider description for FeverUp',
	'https://provider.code-challenge.feverup.com/api/events',
	TRUE
);
